use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;
use std::{env, fs};
use std::path::Path;

fn main() {
	// Init build paths
	let manifest_dir_path =
        env::var("CARGO_MANIFEST_DIR").expect("Missing CARGO_MANIFEST_DIR environment variable");
    let manifest_dir = Path::new(&manifest_dir_path);
    let current_dir = env::current_dir().expect("Couldn't get current directory");
	let source_dir = current_dir.join("src");
    let target_dir_rel = manifest_dir.join("target");
    let target_dir = current_dir.join(target_dir_rel);

	//1. BootLoader
	let bootloader_src = source_dir.join("boot");
	build_bootloader(&bootloader_src, &target_dir);

	//2. x86 Kernel
	build_kernel(&source_dir, &target_dir, "x86");

	//3. AMD64(x86_64) kernel
	// build_kernel(&source_dir, &target_dir, "x86_64");

}

fn build_bootloader(
	source: &Path,
	target: &Path
) {
	println!("cargo:rerun-if-changed={}", &source.display());
	let nasm = Command::new("nasm")
								.arg("-o")
								.arg(format!("{}/BootLoader.bin", &target.display()))
								.arg(format!("{}/BootLoader.asm", &source.display()))
								.status().unwrap();
	assert!(nasm.success());
}

fn build_kernel(
	source: &Path,
	root_target: &Path,
	arch: &str
) {
    println!("cargo:rerun-if-changed={}/arch/{}/src", &source.display(), arch);
	let source = source.join("arch").join(arch);
	let src = source.join("src");
	let target = root_target.join(arch);
	match fs::create_dir(&target) {
		Ok(_) => {},
		Err(_) => {},
	}

	let file_list = fs::read_dir(&src).expect("There are no 'arch/x86' directory")
								.map(|entry| entry.unwrap().file_name());
	let mut entry_file = Vec::<String>::new();
	// let mut rust_file = Vec::<String>::new();
	let mut asm_file = Vec::<String>::new();
	let mut obj_file = Vec::<String>::new();
	for entry in file_list {
		let file_name = entry.to_str().unwrap().to_string();
		let ext: Vec<&str> = file_name.split(".").collect();

		if ext.len() < 2 {
			continue;
		}

		match ext[1] {
			"s" 	=> entry_file.push(format!("{}/{}", src.display(), file_name)),
			// "rs" 	=> rust_file.push(format!("{}/{}", src.display(), file_name)),
			"asm"	=> asm_file.push(format!("{}/{}", src.display(), file_name)),
			_ => {}
		}
	}
	
	if entry_file.len() > 1 {
		panic!("EntryPoint cannot be two");
	}

	let nasm = Command::new("nasm")
				.arg("-o").arg(format!("{}/EntryPoint.bin", &target.display()))
				.arg(&entry_file[0])
				.status().unwrap();
	assert!(nasm.success());

	for file in asm_file {
		let ext: Vec<&str> = file.split(".").collect();
		let nasm = Command::new("nasm")
				.arg("-f").arg("elf32")
				.arg("-o").arg(format!("{}/{}.o", &target.display(), &ext[0]))
				.arg(&file)
				.status().unwrap();
		assert!(nasm.success());
		obj_file.push(ext[0].to_string());
	}

	let cargo = Command::new("xargo").current_dir(&src)
				.arg("build")
				// .arg("-Zbuild-std=core,compiler_builtins")
    			// .arg("--target").arg(format!("{}/triple.json", &source.display()))
				.arg(format!("--target-dir={}", &target.display()))
				.status().unwrap();
	assert!(cargo.success());

	let lib_path = target.join("triple").join("debug");

	// panic!(format!("{}/lib{}.a", lib_path.display(), arch));

	let ld = Command::new("ld")
			.args(["--gc-sections", "-melf_i386", "-nostdlib"])
			.arg("-n")
			.arg("-T").arg(format!("{}/linker.ld", source.display()))
			.arg("-o").arg(format!("{}/Kernel{}.elf", target.display(), arch))
			.arg(format!("{}/lib{}.a", lib_path.display(), arch))//.arg(obj_file.join(".o "))
			.status().unwrap();
	assert!(ld.success());

	let objcopy = Command::new("objcopy")
			.args(["-j", ".text"])
			.args(["-j", ".data"])
			.args(["-j", ".rodata"])
			.args(["-j", ".bss"])
			.args(["-S", "-O", "binary"])
			.arg(format!("{}/Kernel{}.elf", target.display(), arch))
			.arg(format!("{}/Kernel{}.elf.bin", target.display(), arch))
			.status().unwrap();
    assert!(objcopy.success());

    merge_file(vec![&format!("{}/EntryPoint.bin", target.display()),
            &format!("{}/Kernel{}.elf.bin", target.display(), arch)],
            &format!("{}/Kernel{}.bin", root_target.display(), arch));
}

fn merge_file(
    file_list: Vec<&str>,
    output_name: &str
) {
    let mut output = File::create(output_name).unwrap();
    let mut buffer = [0u8; 4096];
    for file_name in file_list {
        let mut file = File::open(file_name).unwrap();

        loop {
            let nbyte = file.read(&mut buffer).unwrap();
            output.write(&buffer[..nbyte]).unwrap();
            if nbyte < buffer.len() { break; }
        }
    }

}
