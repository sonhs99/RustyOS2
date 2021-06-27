use core::u32;

extern "C" {
	pub fn kReadCPUID(
		dwEAX: u32,
		pdwEAX: *mut u32,
		pdwEBX: *mut u32,
		pdwECX: *mut u32,
		pdwEDX: *mut u32,
	);
	pub fn kSwitchAndExecute64BitKernel();
}