#![no_std] // don't link the Rust standard library

#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(asm)]

use core::panic::PanicInfo;

pub mod entry;
pub mod assembly;
pub mod keyboard;
pub mod descriptor;
pub mod utility;
pub mod interrupt;
pub mod pic;
pub mod types;
pub mod console;
pub mod shell;

#[no_mangle] // don't mangle the name of this function
pub unsafe extern "C" fn Main() -> ! {
    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
	asm!(
			"mov ax, 0x10
			mov ds, ax
			mov es, ax
			mov fs, ax
			mov gs, ax

			mov ss, ax
			mov rsp, 0x6FFFF8
			mov rbp, 0x6FFFF8"
		);

	entry::entry();
	
	loop {}
}

pub fn print_string(x:i32, y:i32, str: &[u8]) {
	let vga_buffer = 0xB8000 as *mut u8;
	let base = ((y * 80 + x) * 2) as isize;

	for (i, &byte) in str.iter().enumerate() {
		unsafe {
			*vga_buffer.offset(i as isize * 2 + base) = byte;
		}
	}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	print_string(0, 24, b"[PANIC] Panicked from x64 Kernel");
    loop {}
}