use core::hint::black_box;

use crate::{
    assembly::{DisableInterrupt, EnableInterrupt, ReadRFLAGS},
    println,
};

pub fn memset(dest: *mut u8, data: u8, size: isize) {
    for i in 0..size {
        unsafe {
            *dest.offset(i) = data;
        }
    }
}

pub fn memcpy(dest: *mut u8, src: *const u8, size: isize) -> isize {
    let mut count = 0;
    for i in 0..size {
        unsafe {
            *dest.offset(i) = *src.offset(i);
        }
        count += 1;
    }
    return count;
}

pub fn memcmp(dest: *mut u8, src: *mut u8, size: isize) -> bool {
    for i in 0..size {
        unsafe {
            if *dest.offset(i) == *src.offset(i) {
                return false;
            }
        }
    }
    return true;
}

pub fn set_interrupt_flag(enable_interrupt: bool) -> bool {
    let rflags = ReadRFLAGS();
    if enable_interrupt {
        EnableInterrupt();
    } else {
        DisableInterrupt();
    }

    (rflags & 0x0200) != 0
}

static mut TOTAL_RAM_SIZE: u64 = 0;

pub fn check_ram_size() {
    let mut priv_value;
    const MAGIC_NUMBER: u32 = 0xDEAD_BEEF;
    unsafe {
        let mut address: *mut u32 = 0x400_0000 as *mut u32;
        loop {
            priv_value = *address;

            *address = MAGIC_NUMBER;
            println!("{:X}", *address);
            if *address != MAGIC_NUMBER {
                break;
            }

            *address = priv_value;
            address = address.offset(0x10_0000);
        }
        TOTAL_RAM_SIZE = address as u64 / 0x10_0000;
    }
}

pub fn get_ram_size() -> u64 {
    unsafe { TOTAL_RAM_SIZE }
}
