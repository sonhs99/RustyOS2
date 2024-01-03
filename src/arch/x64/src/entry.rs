use crate::{
    assembly::{self, EnableInterrupt},
    console, descriptor, keyboard,
    pic::{InitializePIC, MaskedPICInterrupt},
    println,
    shell::start_shell,
    utility::{check_ram_size, get_ram_size},
};

#[allow(unconditional_panic)]
pub fn entry() {
    console::init_console(0, 10);
    println!("Swtich to IA-32e Mode.......................[Pass]");
    println!("IA-32e Rust Kernel Start....................[Pass]");

    let (_, mut y) = console::get_curser();

    println!("GDT Initialize And Switch For IA-32e Mode...[    ]");
    descriptor::InitializeGDTTableAndTTS();
    assembly::LoadGDTR(descriptor::GDTR_STARTADDRESS);
    console::set_curser(45, y);
    y += 1;
    println!("Pass");

    println!("TSS Segment Load............................[    ]");
    assembly::LoadTR(descriptor::GDT_TSSSEGMENT);
    console::set_curser(45, y);
    y += 1;
    println!("Pass");

    println!("IDT Initialize..............................[    ]");
    descriptor::InitializeIDTTables();
    assembly::LoadIDTR(descriptor::IDTR_STARTADDRESS);
    console::set_curser(45, y);
    y += 1;
    println!("Pass");

    println!("Total RAM Size Check........................[    ]");
    check_ram_size();
    console::set_curser(45, y);
    y += 1;
    println!("Pass], {} MB", get_ram_size());

    println!("Keyboard Activate And Queue Initialize......[    ]");
    console::set_curser(45, y);
    if keyboard::InitializeKeyboard() {
        println!("Pass");
        keyboard::ChangeKeyboardLED(false, false, false);
    } else {
        println!("Fail");
        loop {}
    }
    y += 1;

    println!("PIC Controller And Interrupt Initialize.....[    ]");
    InitializePIC();
    MaskedPICInterrupt(0);
    EnableInterrupt();
    console::set_curser(45, y);
    println!("Pass");

    start_shell();
}
