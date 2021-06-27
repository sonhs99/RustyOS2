use crate::{assembly, descriptor, keyboard, print_string};

#[allow(unconditional_panic)]
pub fn entry() {
	let mut vcTemp = 0;
	let mut flags = 0;
	let mut i = 0;

	print_string(0, 10, b"Swtich to IA-32e Mode.......................[Pass]");
	print_string(0, 11, b"IA-32e Rust Kernel Start....................[Pass]");

	print_string(0, 12, b"GDT Initialize And Switch For IA-32e Mode...[    ]");
	descriptor::InitializeGDTTableAndTTS();
	assembly::LoadGDTR(descriptor::GDTR_STARTADDRESS);
	print_string(45, 12, b"Pass");

	print_string(0, 13, b"TSS Segment Load............................[    ]");
	assembly::LoadTR(descriptor::GDT_TSSSEGMENT);
	print_string(45, 13, b"Pass");

	print_string(0, 14, b"IDT Initialize..............................[    ]");
	assembly::LoadIDTR(descriptor::IDTR_STARTADDRESS);
	print_string(45, 14, b"Pass");

	print_string(0, 15, b"Keyboard Activate...........................[    ]");
	if keyboard::ActiveKeyboard() {
		print_string(45, 15, b"Pass");
		keyboard::ChangeKeyboardLED(false, false, false);
	} else {
		print_string(45, 15, b"Fail");
		loop {}
	}
	loop {
		if keyboard::IsOutputBufferFull() {
			let temp = keyboard::GetKeyboardScanCode();
			if keyboard::ConvertScanCodeToASCIICode(temp, &mut vcTemp,&mut flags) {
				if (flags & keyboard::KeyStatement::KeyFlagsDown as u8) != 0 {
					print_string(i, 16, &[vcTemp]);

					if vcTemp == '0' as u8 { 1 / 0; }

					i += 1;
				}
			}
		}
	}
}