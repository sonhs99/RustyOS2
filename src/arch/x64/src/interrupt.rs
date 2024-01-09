use crate::{
    keyboard::{ConvertScanCodeAndPutQueue, GetKeyboardScanCode, IsOutputBufferFull},
    pic::{self, SendEOI},
    print_string, process,
    utility::set_interrupt_flag,
};

pub extern "x86-interrupt" fn divided_by_zero() {
    CommonExceptionHandler(0);
}
pub extern "x86-interrupt" fn debug() {
    CommonExceptionHandler(1);
}
pub extern "x86-interrupt" fn NMI() {
    CommonExceptionHandler(2);
}
pub extern "x86-interrupt" fn break_point() {
    CommonExceptionHandler(3);
}
pub extern "x86-interrupt" fn overflow() {
    CommonExceptionHandler(4);
}
pub extern "x86-interrupt" fn bound_range_exceeded() {
    CommonExceptionHandler(5);
}
pub extern "x86-interrupt" fn invalid_opcode() {
    CommonExceptionHandler(6);
}
pub extern "x86-interrupt" fn device_not_avalidable() {
    CommonExceptionHandler(7);
}
pub extern "x86-interrupt" fn double_fault() {
    CommonExceptionHandler(8);
}
pub extern "x86-interrupt" fn coprocessor_segment_overrun() {
    CommonExceptionHandler(9);
}
pub extern "x86-interrupt" fn invalid_tss() {
    CommonExceptionHandler(10);
}
pub extern "x86-interrupt" fn segment_not_present() {
    CommonExceptionHandler(11);
}
pub extern "x86-interrupt" fn stack_segment_fault() {
    CommonExceptionHandler(12);
}
pub extern "x86-interrupt" fn general_protection() {
    CommonExceptionHandler(13);
}
pub extern "x86-interrupt" fn page_fault() {
    CommonExceptionHandler(14);
}
pub extern "x86-interrupt" fn ISR15() {
    CommonExceptionHandler(15);
}
pub extern "x86-interrupt" fn FPU_error() {
    CommonExceptionHandler(16);
}
pub extern "x86-interrupt" fn alignment_check() {
    CommonExceptionHandler(17);
}
pub extern "x86-interrupt" fn machine_check() {
    CommonExceptionHandler(18);
}
pub extern "x86-interrupt" fn SMID_error() {
    CommonExceptionHandler(19);
}
pub extern "x86-interrupt" fn common_exception() {
    CommonExceptionHandler(20);
}

#[naked]
pub fn timer() {
    use core::arch::asm;
    unsafe {
        asm!(
        "push rbp",
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rdi",
        "push rsi",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "mov ax, ds",
        "push rax",
        "mov ax, es",
        "push rax",
        "mov ax, fs",
        "push rax",
        "mov ax, gs",
        "push rax",
        
        "mov rdi, 32",
        "call {func}",

        "pop rax",
        "mov gs, ax",
        "pop rax",
        "mov fs, ax",
        "pop rax",
        "mov es, ax",
        "pop rax",
        "mov ds, ax",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rsi",
        "pop rdi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",
        "pop rbp",
        "iretq",
        func = sym TimerHandler,
        options(noreturn));
    }
}
pub extern "x86-interrupt" fn keyboard() {
    KeyboardHandler(33);
}
pub extern "x86-interrupt" fn slave_PIC() {
    CommonInterruptHandler(34);
}
pub extern "x86-interrupt" fn serial_2() {
    CommonInterruptHandler(35);
}
pub extern "x86-interrupt" fn serial_1() {
    CommonInterruptHandler(36);
}
pub extern "x86-interrupt" fn parallel_2() {
    CommonInterruptHandler(37);
}
pub extern "x86-interrupt" fn floppy() {
    CommonInterruptHandler(38);
}
pub extern "x86-interrupt" fn parallel_1() {
    CommonInterruptHandler(39);
}
pub extern "x86-interrupt" fn rtc() {
    CommonInterruptHandler(40);
}
pub extern "x86-interrupt" fn ISR41() {
    CommonInterruptHandler(41);
}
pub extern "x86-interrupt" fn not_use_0() {
    CommonInterruptHandler(42);
}
pub extern "x86-interrupt" fn not_use_1() {
    CommonInterruptHandler(43);
}
pub extern "x86-interrupt" fn mouse() {
    CommonInterruptHandler(44);
}
pub extern "x86-interrupt" fn coprocessor() {
    CommonInterruptHandler(45);
}
pub extern "x86-interrupt" fn hdd1() {
    CommonInterruptHandler(46);
}
pub extern "x86-interrupt" fn hdd2() {
    CommonInterruptHandler(47);
}
pub extern "x86-interrupt" fn common_interrupt() {
    CommonInterruptHandler(48);
}

fn CommonExceptionHandler(vector: u8) {
    let buffer: [u8; 2] = [vector / 10 + '0' as u8, vector % 10 + '0' as u8];
    print_string(
        0,
        0,
        b"====================================================",
    );
    print_string(
        0,
        1,
        b"               Exception Occur                      ",
    );
    print_string(
        0,
        2,
        b"                  Vector :                          ",
    );
    print_string(27, 2, &buffer);
    print_string(
        0,
        3,
        b"====================================================",
    );
}

fn CommonInterruptHandler(vector: u8) {
    let mut buffer = b"[INT:  , ]".clone();
    static mut common_count: u8 = 0;
    buffer[5] = vector / 10 + '0' as u8;
    buffer[6] = vector % 10 + '0' as u8;
    unsafe {
        buffer[8] = common_count + '0' as u8;
        common_count = (common_count + 1) % 10;
    }
    print_string(70, 0, &buffer);
    SendEOI((vector - pic::PIC_IRQSTARTVECTOR) as u16);
}

fn TimerHandler(vector: u8) {
    let mut buffer = b"[INT:  , ]".clone();
    static mut common_count: u8 = 0;
    buffer[5] = vector / 10 + '0' as u8;
    buffer[6] = vector % 10 + '0' as u8;
    unsafe {
        buffer[8] = common_count + '0' as u8;
        common_count = (common_count + 1) % 10;
    }
    print_string(70, 0, &buffer);

    SendEOI((vector - pic::PIC_IRQSTARTVECTOR) as u16);

    process::decrease_time();
    if process::is_expired() {
        process::schedule();
    }
}

fn KeyboardHandler(vector: u8) {
    let mut buffer = b"[INT:  , ]".clone();
    static mut keyboard_count: u8 = 0;
    buffer[5] = vector / 10 + '0' as u8;
    buffer[6] = vector % 10 + '0' as u8;
    unsafe {
        buffer[8] = keyboard_count + '0' as u8;
        keyboard_count = (keyboard_count + 1) % 10;
    }
    print_string(0, 0, &buffer);

    if IsOutputBufferFull() {
        let temp = GetKeyboardScanCode();
        ConvertScanCodeAndPutQueue(temp);
    }

    SendEOI((vector - pic::PIC_IRQSTARTVECTOR) as u16);
}

pub fn without_interrupt<F>(mut f: F)
where
    F: FnMut(),
{
    let previous_flag = set_interrupt_flag(false);
    f();
    set_interrupt_flag(previous_flag);
}
