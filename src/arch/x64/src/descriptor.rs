#![allow(dead_code)]
#![allow(non_snake_case)]

use core::mem::size_of;
use crate::{utility::memset, print_string};

const GDT_TYPE_CODE			:u8 = 0x0A;
const GDT_TYPE_DATA			:u8 = 0x02;
const GDT_TYPE_TSS			:u8 = 0x09;
const GDT_FLAGS_LOWER_S		:u8 = 0x10;
const GDT_FLAGS_LOWER_DPL0	:u8 = 0x00;
const GDT_FLAGS_LOWER_DPL1	:u8 = 0x20;
const GDT_FLAGS_LOWER_DPL2	:u8 = 0x40;
const GDT_FLAGS_LOWER_DPL3	:u8 = 0x60;
const GDT_FLAGS_LOWER_P		:u8 = 0x80;
const GDT_FLAGS_UPPER_L		:u8 = 0x20;
const GDT_FLAGS_UPPER_DB	:u8 = 0x40;
const GDT_FLAGS_UPPER_G		:u8 = 0x80;

const GDT_FLAG_LOWER_KERNELCODE	:u8 = GDT_TYPE_CODE | GDT_FLAGS_LOWER_S | GDT_FLAGS_LOWER_DPL0 | GDT_FLAGS_LOWER_P;
const GDT_FLAG_LOWER_KERNELDATA	:u8 = GDT_TYPE_DATA | GDT_FLAGS_LOWER_S | GDT_FLAGS_LOWER_DPL0 | GDT_FLAGS_LOWER_P;
const GDT_FLAG_LOWER_TSS		:u8 = 									  GDT_FLAGS_LOWER_DPL0 | GDT_FLAGS_LOWER_P;
const GDT_FLAG_LOWER_USERCODE	:u8 = GDT_TYPE_CODE | GDT_FLAGS_LOWER_S | GDT_FLAGS_LOWER_DPL3 | GDT_FLAGS_LOWER_P;
const GDT_FLAG_LOWER_USERDATA	:u8 = GDT_TYPE_CODE | GDT_FLAGS_LOWER_S | GDT_FLAGS_LOWER_DPL3 | GDT_FLAGS_LOWER_P;

const GDT_FLAGS_UPPER_CODE:u8 = GDT_FLAGS_UPPER_G | GDT_FLAGS_UPPER_L;
const GDT_FLAGS_UPPER_DATA:u8 = GDT_FLAGS_UPPER_G | GDT_FLAGS_UPPER_L;
const GDT_FLAGS_UPPER_TSS :u8 = GDT_FLAGS_UPPER_G;

pub const GDT_KERNELCODESEGMENT	:u16 = 0x08;
pub const GDT_KERNELDATASEGMENT	:u16 = 0x10;
pub const GDT_TSSSEGMENT		:u16 = 0x18;

pub const GDTR_STARTADDRESS	:u64 = 0x142000;
const GDT_MAXENTRY8COUNT:u32 = 3;
const GDT_MAXENTRY16COUNT:u32 = 1;

const GDT_TABLESIZE		:u32 = (size_of::<GDT8ENTRY>() as u32) * GDT_MAXENTRY8COUNT +
								(size_of::<GDT16ENTRY>() as u32) * GDT_MAXENTRY16COUNT;
const TSS_SEGMENTSIZE	:u32 = size_of::<TSSSEGMENT>() as u32;

const IDT_TYPE_INTERRUPT:u8 = 0x0E;
const IDT_TYPE_TRAP		:u8 = 0x0F;
const IDT_FLAGS_DPL0	:u8 = 0x00;
const IDT_FLAGS_DPL1	:u8 = 0x20;
const IDT_FLAGS_DPL2	:u8 = 0x40;
const IDT_FLAGS_DPL3	:u8 = 0x60;
const IDT_FLAGS_P		:u8 = 0x80;
const IDT_FLAGS_IST0	:u8 = 0;
const IDT_FLAGS_IST1	:u8 = 1;

const IDT_FLAGS_KERNEL	:u8 = IDT_FLAGS_DPL0 | IDT_FLAGS_P;
const IDT_FLAGS_USER	:u8 = IDT_FLAGS_DPL3 | IDT_FLAGS_P;

const IDT_MAXENTRYCOUNT	:u16 = 100;
pub const IDTR_STARTADDRESS	:u64 = GDTR_STARTADDRESS + (size_of::<GDTR>() as u64) + GDT_TABLESIZE as u64 + TSS_SEGMENTSIZE as u64;
const IDT_STARTADDRESS	:u64 = IDTR_STARTADDRESS + (size_of::<IDTR>() as u64);
const IDT_TABLESIZE		:u16 = IDT_MAXENTRYCOUNT * (size_of::<IDTENTRY>() as u16);

const IST_STARTADDRESS	:u32 = 0x700000;
const IST_SIZE			:u32 = 0x100000;

#[repr(C, packed(1))]
struct GDTRStruct {
	Limit: u16,
	BaseAddress: u64,
	Padding1: u16,
	Padding2: u32,
}

#[repr(C, packed(1))]
struct GDTEntry8Struct {
	LowerLimit: u16,
	LowerBaseAddress: u16,
	UpperBaseAddress1: u8,
	TypeAndLowerFlag: u8,
	UpperLimitAndUpperFlag: u8,
	UpperBaseAddress2: u8
}

#[repr(C, packed(1))]
struct GDTEntry16Struct {
	LowerLimit: u16,
	LowerBaseAddress: u16,
	MiddleBaseAddress1: u8,
	TypeAndLowerFlag: u8,
	UpperLimitAndUpperFlag: u8,
	MiddleBaseAddress2: u8,
	UpperBaseAddress: u32,
	Reserved: u32
}

#[repr(C, packed(1))]
struct TSSDataStruct {
	dwReserved1: u32,
	Rsp: [u64; 3],
	qwReserved2: u64,
	IST: [u64; 7],
	qwReserved3: u64,
	wReserved: u16,
	IOMapBaseAddress: u16
}

#[repr(C, packed(1))]
struct IDTEntryStruct {
	LowerBaseAddress: u16,
	SegmentSelector: u16,
	IST: u8,
	TypeAndFlags: u8,
	MiddleBaseAddress: u16,
	UpperBaseAddress: u32,
	Reserved: u32
}

type GDTR = GDTRStruct;
type IDTR = GDTRStruct;
type GDT8ENTRY = GDTEntry8Struct;
type GDT16ENTRY = GDTEntry16Struct;
type TSSSEGMENT = TSSDataStruct;
type IDTENTRY = IDTEntryStruct;

impl GDTEntry8Struct {
	pub fn set(
		&mut self,
		BaseAddress: u32,
		Limit: u32,
		UpperFlags: u8,
		LowerFlags: u8,
		Type: u8
	) {
		self.LowerLimit = Limit as u16 & 0xFFFF;
		self.LowerBaseAddress = BaseAddress as u16 & 0xFFFF;
		self.UpperBaseAddress1 = ((BaseAddress >> 16) & 0xFF) as u8;
		self.TypeAndLowerFlag = LowerFlags | Type;
		self.UpperLimitAndUpperFlag = ((Limit >> 16) & 0xFF ) as u8 | UpperFlags;
		self.UpperBaseAddress2 = ((BaseAddress >> 24) & 0xFF) as u8;
	}
}

impl GDTEntry16Struct {
	pub fn set(
		&mut self,
		BaseAddress: u64,
		Limit: u32,
		UpperFlags: u8,
		LowerFlags: u8,
		Type: u8
	) {
		self.LowerLimit = Limit as u16 & 0xFFFF;
		self.LowerBaseAddress = BaseAddress as u16 & 0xFFFF;
		self.MiddleBaseAddress1 = ((BaseAddress >> 16) & 0xFF) as u8;
		self.TypeAndLowerFlag = LowerFlags | Type;
		self.UpperLimitAndUpperFlag = ((Limit >> 16) & 0xFF ) as u8 | UpperFlags;
		self.MiddleBaseAddress2 = ((BaseAddress >> 24) & 0xFF) as u8;
		self.UpperLimitAndUpperFlag = (BaseAddress >> 32) as u8;
		self.Reserved = 0;
	}
}

impl IDTEntryStruct {
	pub fn set(
		&mut self,
		handler: u64,
		selector: u16,
		IST: u8,
		flags: u8,
		Type: u8
	) {
		self.LowerBaseAddress = (handler & 0xFFFF) as u16;
		self.SegmentSelector = selector;
		self.IST = IST & 0x3;
		self.TypeAndFlags = Type | flags;
		self.MiddleBaseAddress = ((handler >> 16) & 0xFFFF) as u16;
		self.UpperBaseAddress = (handler >> 32) as u32;
		self.Reserved = 0;
	}
}

pub fn InitializeGDTTableAndTTS() {
	let pGDTR = GDTR_STARTADDRESS as *mut GDTR;
	let pEntry = (GDTR_STARTADDRESS + (size_of::<GDTR>() as u64)) as *mut GDT8ENTRY;
	let pTSS = ((pEntry as u64) + (GDT_TABLESIZE as u64)) as *mut TSSSEGMENT;
	unsafe {
		(*pGDTR).Limit = GDT_TABLESIZE as u16 - 1;
		(*pGDTR).BaseAddress = pEntry as u64;
		(*pEntry.offset(0)).set(0, 0, 0, 0, 0);
		(*pEntry.offset(1)).set(0, 0xFFFFF, GDT_FLAGS_UPPER_CODE, GDT_FLAG_LOWER_KERNELCODE, GDT_TYPE_CODE);
		(*pEntry.offset(2)).set(0, 0xFFFFF, GDT_FLAGS_UPPER_DATA, GDT_FLAG_LOWER_KERNELDATA, GDT_TYPE_DATA);
		(*((pEntry.offset(3) as u64) as *mut GDT16ENTRY)).set( 
			pTSS as u64, 
			size_of::<TSSSEGMENT>() as u32 - 1, 
			GDT_FLAGS_UPPER_TSS,
			GDT_FLAG_LOWER_TSS, 
			GDT_TYPE_TSS
		);

		InitializeTTSSegment( pTSS );
	}
}

fn InitializeTTSSegment( pTSS: *mut TSSSEGMENT) {
	memset(pTSS as *mut u8, 0, size_of::<TSSSEGMENT>() as isize);
	unsafe {
		(*pTSS).IST[0] = (IST_STARTADDRESS + IST_SIZE) as u64;
		(*pTSS).IOMapBaseAddress = 0xFFFF;
	}
}

pub fn InitializeIDTTables() {
	let pIDTR = IDTR_STARTADDRESS as *mut IDTR;
	let pEntry = (IDTR_STARTADDRESS + size_of::<IDTR>() as u64) as *mut IDTENTRY;

	unsafe {
		(*pIDTR).BaseAddress = pEntry as u64;
		(*pIDTR).Limit = IDT_TABLESIZE - 1;

		for i in 0..IDT_MAXENTRYCOUNT {
			(*pEntry.offset(i as isize)).set(
				DummyHandler as u64, 0x08, IDT_FLAGS_IST1, IDT_FLAGS_KERNEL, IDT_TYPE_INTERRUPT
			)
		}
	}
}

fn DummyHandler(){
	print_string( 0, 0, b"====================================================" );
    print_string( 0, 1, b"          Dummy Interrupt Handler Execute~!!!       " );
    print_string( 0, 2, b"           Interrupt or Exception Occur~!!!!        " );
    print_string( 0, 3, b"====================================================" );
	loop{}
}