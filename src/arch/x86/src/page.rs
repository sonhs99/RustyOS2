#![allow(non_snake_case)]
#![allow(dead_code)]

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
    pub fn set(
		&mut self,
        UpperBaseAddress: u32,
        LowerBaseAddress: u32,
        LowerFlags: u32,
        UpperFlags: u32
    ) {
		self.AttributeAndLowerBaseAddress = LowerBaseAddress | LowerFlags;
        self.UpperBaseAddressAndEXB = ( UpperBaseAddress & 0xFF ) | UpperFlags;
    }
}

pub unsafe fn InitPageTable() {
	let PML4Entry = 0x100000 as *mut PML4ENTRY;
    (*PML4Entry.offset(0)).set(0x0, 0x101000, PAGE_FLAGS_DEFAULT, 0);
    for i in 1..PAGE_MAXENTRYCOUNT {
        (*PML4Entry.offset(i as isize)).set(0x0, 0x0, 0, 0);
    }

	let PDPTEntry = 0x101000 as *mut PDPTENTRY;
    for i in 0..64 {
        (*PDPTEntry.offset(i as isize)).set(0x0, 0x102000 + ( i * PAGE_TABLESIZE ), PAGE_FLAGS_DEFAULT, 0);
    }
    for i in 64..PAGE_MAXENTRYCOUNT {
        (*PDPTEntry.offset(i as isize)).set(0x0, 0x0, 0, 0);
    }

	let PDEntry = 0x102000 as *mut PDENTRY;
    let mut MappingAddress: u32 = 0;
	let size = PAGE_MAXENTRYCOUNT * 64;
    for i in 0..size {
        (*PDEntry.offset(i as isize))
            	.set((i * ( PAGE_DEFAULTSIZE >> 20 ) ) >> 12, MappingAddress, PAGE_FLAGS_DEFAULT | PAGE_FLAGS_PS, 0);
        MappingAddress += PAGE_DEFAULTSIZE;
    }
	
}