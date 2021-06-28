#![no_std] // don't link the Rust standard library

#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]

#![allow(non_snake_case)]

pub mod page;
pub mod mode_switch;
use core::panic::PanicInfo;

use mode_switch::{kReadCPUID, kSwitchAndExecute64BitKernel};

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
	print_string(0, 3, b"Rust Kernel Start...........................[Pass]");
	
	print_string(0, 4, b"Minimum Memory Check........................[    ]");
	if is_memory_enough() {
		print_string(45, 4, b"Pass");
	} else {
		print_string(45, 4, b"Fail");
		print_string(0, 5, b"[ERROR] Not Enough Memory. Require Over 64M");
		loop {}
	}

	print_string(0, 5, b"IA-32e Kernel Area Initialization...........[    ]");
	if init_kernel64_area() {
		print_string(45, 5, b"Pass");
	} else {
		print_string(45, 5, b"Fail");
		print_string(0, 6, b"[ERROR] Kernel Area Initialization Failed");
		loop {}
	}

	print_string(0, 6, b"IA-32e Page Tables Initialize...............[    ]");
	unsafe { page::InitPageTable(); }
	print_string(45, 6, b"Pass");

	let (mut EAX, mut EBX, mut ECX, mut EDX): (u32, u32, u32, u32) = (0, 0, 0, 0);
	unsafe { kReadCPUID(0x00, &mut EAX, &mut EBX, &mut ECX, &mut EDX); }
	print_string(0, 7, b"Proccessor Vendor String....................[            ]");
	print_string(45, 7, &u32_to_u8_array(EBX));
	print_string(49, 7, &u32_to_u8_array(EDX));
	print_string(53, 7, &u32_to_u8_array(ECX));

	unsafe { kReadCPUID(0x00, &mut EAX, &mut EBX, &mut ECX, &mut EDX); }
	print_string(0, 8, b"64bit Mode Support Check....................[    ]");
	if (EDX & ( 1 << 29 )) != 0 {
		print_string(45, 8, b"Pass");
	} else {
		print_string(45, 8, b"Fail");
		print_string(0, 9, b"[ERROR] This processor does not support 64bit mode");
		loop {}
	}

	print_string(0, 9, b"Copy IA-32e Kernel To 2M Address............[    ]");
	copy_kernel64_image_to_2mbyte();
	print_string(45, 9, b"Pass");

	print_string(0, 10, b"Switch to IA-32e Mode");
	unsafe { kSwitchAndExecute64BitKernel(); }
    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	print_string(0, 24, b"[PANIC] Panicked from x86 Kernel");
    loop {}
}

fn print_string(x:i32, y:i32, str: &[u8]) {
	let vga_buffer = 0xB8000 as *mut u8;
	let base = ((y * 80 + x) * 2) as isize;

	for (i, &byte) in str.iter().enumerate() {
		unsafe {
			*vga_buffer.offset(i as isize * 2 + base) = byte;
		}
	}
}

fn init_kernel64_area() -> bool {
	let base_address = 0x100000 as *mut u32;
	let size: u32 = 0x600000 - (base_address as u32); 
	for i in 0..(size/4) {
		unsafe {
			*base_address.offset(i as isize) = 0x00;
			if *base_address.offset(i as isize) != 0x00 {
				return false;
			}
		}
	}
	return true;
}

fn is_memory_enough() -> bool {
	let base_address = 0x100000 as *mut u32;
	let size: u32 = 0x4000000 - (base_address as u32);
	for i in 0..(size/4) {
		unsafe {
			*base_address.offset(i as isize) = 0xFEEBFEEB;
			if *base_address.offset(i as isize) != 0xFEEBFEEB {
				return false;
			}
		}
	}
	return true;
}

fn copy_kernel64_image_to_2mbyte() {
	let total_kernel_sector_count: u16 = unsafe { *( 0x7c05 as *mut u16 ) };
	let kernel32_sector_count: u16 = unsafe { *( 0x7c07 as *mut u16 ) };


	let source_address = (0x10000 + (kernel32_sector_count as u32) * 512 ) as *mut u32;
	let destination_address = 0x200000 as *mut u32;

	let size = 512 * ( total_kernel_sector_count - kernel32_sector_count ) / 4;
	let mut data: u32;
	for i in 0..size {
		unsafe { 
			data = *source_address.offset(i as isize);
			*destination_address.offset(i as isize) = data;
		}
	}
}

fn u32_to_u8_array(x: u32) -> [u8; 4] {
  let b1: u8 = ((x >> 24) & 0xff) as u8;
  let b2: u8 = ((x >> 16) & 0xff) as u8;
  let b3: u8 = ((x >> 8) & 0xff) as u8;
  let b4: u8 = (x & 0xff) as u8;
  
  [b4, b3, b2, b1]
}