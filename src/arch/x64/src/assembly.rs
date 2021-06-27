#![allow(non_snake_case)]

pub fn InPortByte(port: u16) -> u8{
	let mut output: u8;
	unsafe {
		asm!(
			"mov rdx, {0}
			 mov rax, 0
			 in al, dx
			 mov {1}, al",
			 in(reg) (port as u64),
			 out(reg_byte) output
		);
	}
	return output;
}

pub fn OutPortByte(port: u16, data: u8) {
	unsafe {
		asm!(
			"mov rdx, {0}
			 mov rax, {1}
			 out dx, al",
			 in(reg) (port as u64),
			 in(reg) (data as u64)
		);
	}
}