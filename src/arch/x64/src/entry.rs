use crate::{assembly::{self, EnableInterrupt}, descriptor, interrupt::keyboard, keyboard, pic::{InitializePIC, MaskedPICInterrupt}, print_string};

#[allow(unconditional_panic)]
pub fn entry() {
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
	descriptor::InitializeIDTTables();
	assembly::LoadIDTR(descriptor::IDTR_STARTADDRESS);
	print_string(45, 14, b"Pass");

	print_string(0, 15, b"Keyboard Activate And Queue Initialize......[    ]");
	if keyboard::InitializeKeyboard() {
		print_string(45, 15, b"Pass");
		keyboard::ChangeKeyboardLED(false, false, false);
	} else {
		print_string(45, 15, b"Fail");
		loop {}
	}

	print_string(0, 16, b"PIC Controller And Interrupt Initialize.....[    ]");
	InitializePIC();
	MaskedPICInterrupt(0);
	EnableInterrupt();
	print_string(45, 16, b"Pass");

	loop {
		let mut key_data: keyboard::KeyData = keyboard::KeyData::new();
		if keyboard::GetKeyFromKeyQueue(&mut key_data) {
			if (key_data.Flags & keyboard::KeyStatement::KeyFlagsDown as u8) != 0 {
				print_string(i, 17, &[key_data.ASCIICode]);
				i += 1;
			}
		}
	}
}