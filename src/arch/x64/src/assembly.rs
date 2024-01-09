#![allow(non_snake_case)]
use core::arch::asm;

pub fn InPortByte(port: u16) -> u8 {
    let mut output: u8;
    unsafe {
        asm!(
            "mov rax, 0
             in al, dx",
             in("rdx") (port as u64),
             out("al") output,
             options(nostack, preserves_flags)
        );
    }
    return output;
}

pub fn OutPortByte(port: u16, data: u8) {
    unsafe {
        asm!(
            "out dx, al",
             in("rdx") (port as u64),
             in("rax") (data as u64),
             options(nostack, preserves_flags)
        );
    }
}

pub fn LoadGDTR(GDTRAddress: u64) {
    unsafe {
        asm!(
            "lgdt [ {0} ]",
            in(reg) GDTRAddress
        );
    }
}

pub fn LoadTR(TSSSegmentOffset: u16) {
    unsafe {
        asm!(
            "ltr {0:x}",
            in(reg) TSSSegmentOffset
        )
    }
}

pub fn LoadIDTR(IDTRAddress: u64) {
    unsafe {
        asm!(
            "lidt [ {0} ]",
            in(reg) IDTRAddress
        );
    }
}

pub fn Int3() {
    unsafe {
        asm!("Int 3");
    }
}

pub fn EnableInterrupt() {
    unsafe {
        asm!("sti");
    }
}

pub fn DisableInterrupt() {
    unsafe {
        asm!("cli");
    }
}

pub fn ReadRFLAGS() -> u64 {
    let mut flag: u64;
    unsafe {
        asm!(
            "pushfq
             pop {0}",
             out(reg) flag
        );
    }
    flag
}

pub fn read_TSC() -> u64 {
    let mut rax: u64;
    let mut rdx: u64;
    unsafe {
        asm!(
            "rdtsc",
            out("rax") rax,
            out("rdx") rdx,
            // options(nostack, preserves_flags)
        );
    }
    rdx << 32 | rax >> 32
}

pub fn halt() {
    unsafe {
        asm!("hlt", "hlt");
    }
}
