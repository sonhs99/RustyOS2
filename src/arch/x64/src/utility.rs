use crate::assembly::{DisableInterrupt, EnableInterrupt, ReadRFLAGS};

pub fn memset(dest: *mut u8, data: u8, size: isize){
	for i in 0..size{
		unsafe { *dest.offset(i) = data; }
	}
}

pub fn memcpy(dest: *mut u8, src: *const u8, size: isize) -> isize {
	let mut count = 0;
	for i in 0..size{
		unsafe { *dest.offset(i) = *src.offset(i); }
		count += 1;
	}
	return count;
}

pub fn memcmp(dest: *mut u8, src: *mut u8, size: isize) -> bool {
	for i in 0..size{
		unsafe { 
			if *dest.offset(i) == *src.offset(i) { return false; }
		}
	}
	return true;
}

pub fn set_interrupt_flag(enable_interrupt: bool) -> bool {
	let rflags = ReadRFLAGS();
	if enable_interrupt { EnableInterrupt(); }
	else { DisableInterrupt(); }

	(rflags & 0x0200) != 0
}