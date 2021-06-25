#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

pub mod page;

use core::panic::PanicInfo;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
	print_string(0, 3, b"Rust Kernel Start...........................[PASS]");

	print_string(0, 4, b"Minimum Memory Check........................[    ]");
	if is_memory_enough() {
		print_string(45, 4, b"PASS");
	} else {
		print_string(45, 4, b"FAIL");
		print_string(0, 5, b"[ERROR] Not Enough Memory. Require Over 64M");
		loop {}
	}

	print_string(0, 5, b"IA-32e Kernel Area Initialization...........[    ]");
	if init_kernel64_area() {
		print_string(45, 5, b"PASS");
	} else {
		print_string(45, 5, b"FAIL");
		print_string(0, 6, b"[ERROR] Kernel Area Initialization Failed");
		loop {}
	}

	print_string(0, 6, b"IA-32e Page Tables Initialize...............[    ]");
	page::init();
	print_string(45, 6, b"PASS");
    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
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
	for i in 0..(500000/4) {
		unsafe {
			*base_address.offset(i * 4) = 0x00;
			if *base_address.offset(i * 4) != 0x00 {
				return false;
			}
		}
	}
	return true;
}

fn is_memory_enough() -> bool {
	let base_address = 0x100000 as *mut u32;
	for i in 0..(300000/4) {
		unsafe {
			*base_address.offset(i * 4) = 0x12345678;
			if *base_address.offset(i * 4) != 0x12345678 {
				return false;
			}
		}
	}
	return true;
}