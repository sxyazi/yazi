use std::{ffi::{c_char, c_int}, ptr::NonNull};

use objc2_core_foundation::{CFAllocator, CFMutableDictionary, CFString, CFType};

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
	pub fn IOServiceGetMatchingServices(
		mainPort: u32,
		matching: *mut CFMutableDictionary,
		existing: *mut u32,
	) -> c_int;

	pub fn IOServiceMatching(a: *const c_char) -> *mut CFMutableDictionary;

	pub fn IOIteratorNext(iterator: u32) -> u32;

	pub fn IORegistryEntryCreateCFProperty(
		entry: u32,
		key: &CFString,
		allocator: Option<&CFAllocator>,
		options: u32,
	) -> Option<NonNull<CFType>>;

	pub fn IOObjectRelease(obj: u32) -> c_int;
}
