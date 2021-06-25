use std::{env, fs::File, io::{Read, Write}, path::Path};
use positioned_io::WriteAt;

static BOOTLOADER: &str = "BootLoader.bin";
static KERNEL32: &str = "Kernelx86.bin";
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
        &format!("{}/{}", target_dir.display(), KERNEL32)
    ], &mut output);

    let bootloader_sector = sector_counts[0];
    let kernel32_sector = sector_counts[1];
    println!("BootLoader    : {} Sectors", bootloader_sector);
    println!("32 Bit Kernel : {} Sectors", kernel32_sector);
    println!("Total         : {} Sectors", bootloader_sector + &kernel32_sector);

    output.write_at(5, &u16_to_u8(kernel32_sector)).unwrap();

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
            for i in nbyte..512 {
                buffer[i] = 0;
            }
            output.write(&buffer[..nbyte]).unwrap();
            count += 1;
            if nbyte < buffer.len() { break; }
        }

        sector_count.push(count);

    }
    sector_count
}