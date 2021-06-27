use crate::{keyboard, print_string};

pub fn entry() {
	let mut vcTemp = 0;
	let mut flags = 0;
	let mut i = 0;

	print_string(0, 10, b"Swtich to IA-32e Mode.......................[Pass]");
	print_string(0, 11, b"IA-32e Rust Kernel Start....................[Pass]");
	print_string(0, 12, b"Keyboard Activate...........................[    ]");
	if keyboard::ActiveKeyboard() {
		print_string(45, 12, b"Pass");
		keyboard::ChangeKeyboardLED(false, false, false);
	} else {
		print_string(45, 12, b"Fail");
		loop {}
	}
	loop {
		if keyboard::IsOutputBufferFull() {
			let temp = keyboard::GetKeyboardScanCode();
			if keyboard::ConvertScanCodeToASCIICode(temp, &mut vcTemp,&mut flags) {
				
				if (flags & keyboard::KeyStatement::KeyFlagsDown as u8) != 0 {
					print_string(i, 13, &[vcTemp]);
					i += 1;
				}
			}
		}
	}
}