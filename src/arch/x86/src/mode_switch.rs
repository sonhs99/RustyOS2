use core::{arch::asm, u32};

extern "C" {
    pub fn kSwitchAndExecute64BitKernel();
}

pub struct CpuidResult {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

pub fn read_cpuid(leaf: u32, sub_leaf: u32) -> CpuidResult {
    let eax;
    let ebx;
    let ecx;
    let edx;

    unsafe {
        asm!(
            "mov edi, ebx",
            "cpuid",
            "xchg edi, ebx",
            out("edi") ebx,
            inout("eax") leaf => eax,
            inout("ecx") sub_leaf => ecx,
            out("edx") edx,
            options(nostack, preserves_flags),
        );
    }

    CpuidResult { eax, ebx, ecx, edx }
}

// pub fn long_mode_switch() {
//     unsafe {
//         asm!(
//             "mov eax, cr4",
//             "or eax, 0x20",
//             "mov cr4, eax",
//             "mov eax, 0x100000",
//             "mov cr3, eax",
//             "mov ecx, 0xC0000080",
//             "rdmsr",
//             "or eax, 0x0100",
//             "wrmsr",
//             "mov eax, cr0",
//             "or eax, 0xE0000000",
//             "xor eax, 0x60000000",
//             "mov cr0, eax",
//             "jmp 0x08:0x200000",
//             options(nostack),
//         );
//     }
// }
