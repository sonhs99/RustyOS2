use std::{env, fs::File, io::{Read, Write}, path::Path};
use positioned_io::WriteAt;

static BOOTLOADER: &str = "BootLoader.bin";
static KERNEL32: &str = "Kernelx86.bin";
static KERNEL64: &str = "Kernelx64.bin";
static DISK: &str = "Disk.img";

#[inline]
fn u16_to_u8(v: u16) -> [u8; 2] {
    [
        v as u8,
        (v >> 8) as u8,
    ]
}

fn main() {
    let manifest_dir_path =
        env::var("CARGO_MANIFEST_DIR").expect("Missing CARGO_MANIFEST_DIR environment variable");
    let manifest_dir = Path::new(&manifest_dir_path);
    let current_dir = env::current_dir().expect("Couldn't get current directory");
    let target_dir_rel = manifest_dir.join("target");
    let target_dir = current_dir.join(target_dir_rel);

    let mut output = File::create(format!("{}/{}", current_dir.display(), DISK)).unwrap();
    
    let sector_counts = merge_kernel(vec![
        &format!("{}/{}", target_dir.display(), BOOTLOADER),
        &format!("{}/{}", target_dir.display(), KERNEL32),
		&format!("{}/{}", target_dir.display(), KERNEL64),
    ], &mut output);

    let bootloader_sector = sector_counts[0];
    let kernel32_sector = sector_counts[1];
	let kernel64_sector = sector_counts[2];
	println!("----------------FILE LIST------------------");
    println!("BootLoader    : {:2} Sector(s), Offset [{:#04x}]", bootloader_sector, 0);
    println!("32-bit Kernel : {:2} Sector(s), Offset [{:#04x}]", kernel32_sector, bootloader_sector);
	println!("64-bit Kernel : {:2} Sector(s), Offset [{:#04x}]", kernel64_sector, bootloader_sector + kernel32_sector);
	println!("----------------SUB TOTAL------------------");
	println!("Total Kernel  : {:2} Sector(s)", kernel32_sector + kernel64_sector);
	println!("------------------TOTAL--------------------");
    println!("Total         : {:2} Sector(s)", bootloader_sector + kernel32_sector + kernel64_sector);

    output.write_at(5, &u16_to_u8(kernel32_sector + kernel64_sector)).unwrap();
	output.write_at(7, &u16_to_u8(kernel32_sector)).unwrap();
}

fn merge_kernel(
    file_list: Vec<&str>,
    output: &mut File
) -> Vec<u16>{
    let mut buffer = [0u8; 512];
    let mut sector_count: Vec<u16> = Vec::new();

    for file_name in file_list {
        let mut file = File::open(file_name).unwrap();
        let mut count: u16 = 0;

        loop {
            let nbyte = file.read(&mut buffer).unwrap();
			if nbyte == 0 { break; }
            for i in nbyte..512 {
                buffer[i] = 0;
            }
            output.write(&buffer).unwrap();
            count += 1;
            if nbyte < buffer.len() { break; }
        }

        sector_count.push(count);

    }
    sector_count
}