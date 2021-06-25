#![allow(non_snake_case)]
const PAGE_FLAGS_P      : u32 = 0x00000001;
const PAGE_FLAGS_RW     : u32 = 0x00000002;
const PAGE_FLAGS_US     : u32 = 0x00000004;
const PAGE_FLAGS_PWT    : u32 = 0x00000008;
const PAGE_FLAGS_PCD    : u32 = 0x00000010;
const PAGE_FLAGS_A      : u32 = 0x00000020;
const PAGE_FLAGS_D      : u32 = 0x00000040;
const PAGE_FLAGS_PS     : u32 = 0x00000080;
const PAGE_FLAGS_G      : u32 = 0x00000100;
const PAGE_FLAGS_PAT    : u32 = 0x00001000;
const PAGE_FLAGS_EXB    : u32 = 0x80000000;
const PAGE_FLAGS_DEFAULT: u32 = PAGE_FLAGS_P | PAGE_FLAGS_RW;
const PAGE_TABLESIZE: u32 = 0x1000;
const PAGE_MAXENTRYCOUNT: u32 = 512;
const PAGE_DEFAULTSIZE: u32 = 0x200000;

#[repr(C, packed(1))]
struct PageTableEntryStruct {
    AttributeAndLowerBaseAddress: u32,
    UpperBaseAddressAndEXB: u32
}

type PML4ENTRY = PageTableEntryStruct;
type PDPTENTRY = PageTableEntryStruct;
type PDENTRY = PageTableEntryStruct;
type PTENTRY = PageTableEntryStruct;

impl PageTableEntryStruct {
    pub fn new(
        UpperBaseAddress: u32,
        LowerBaseAddress: u32,
        LowerFlags: u32,
        UpperFlags: u32
    ) -> Self {
        PageTableEntryStruct{
            AttributeAndLowerBaseAddress: LowerBaseAddress | LowerFlags,
            UpperBaseAddressAndEXB: ( UpperBaseAddress & 0xFF ) | UpperFlags,
        }
    }
}

pub fn init() {
    let PML4Entry = 0x100000 as *mut PML4ENTRY;
    let PDPTEntry = 0x101000 as *mut PDPTENTRY;
    let PDEntry = 0x102000 as *mut PDENTRY;
    let mut MappingAddress: u32 = 0;

    unsafe { *PML4Entry.offset(0) = PML4ENTRY::new(0x0, 0x10100, PAGE_FLAGS_DEFAULT, 0); }
    for i in 1..PAGE_MAXENTRYCOUNT {
        unsafe { *PML4Entry.offset((i * 8) as isize) = PML4ENTRY::new(0x0, 0x0, 0, 0); }
    }

    for i in 0..64 {
        unsafe { *PDPTEntry.offset((i * 8) as isize) = PDPTENTRY::new(0x0, 0x10200 + ( i * PAGE_TABLESIZE ), PAGE_FLAGS_DEFAULT, 0); }
    }
    for i in 64..PAGE_MAXENTRYCOUNT {
        unsafe { *PDPTEntry.offset((i * 8) as isize) = PDPTENTRY::new(0x0, 0x0, 0, 0); }
    }

    for i in 0..(PAGE_MAXENTRYCOUNT * 64) {
        unsafe { *PDEntry.offset((i * 8) as isize) = 
            PDENTRY::new(0x10200 + (i * ( PAGE_DEFAULTSIZE >> 20 ) ) >> 12, MappingAddress, PAGE_FLAGS_DEFAULT | PAGE_FLAGS_PS, 0); }
        MappingAddress += PAGE_DEFAULTSIZE;
    }
}