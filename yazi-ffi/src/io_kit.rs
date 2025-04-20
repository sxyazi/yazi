use std::ffi::c_char;

use core_foundation_sys::{base::{CFAllocatorRef, CFTypeRef, mach_port_t}, dictionary::CFMutableDictionaryRef, string::CFStringRef};
use libc::kern_return_t;

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
	pub fn IOServiceGetMatchingServices(
		mainPort: mach_port_t,
		matching: CFMutableDictionaryRef,
		existing: *mut mach_port_t,
	) -> kern_return_t;

	pub fn IOServiceMatching(a: *const c_char) -> CFMutableDictionaryRef;

	pub fn IOIteratorNext(iterator: mach_port_t) -> mach_port_t;

	pub fn IORegistryEntryCreateCFProperty(
		entry: mach_port_t,
		key: CFStringRef,
		allocator: CFAllocatorRef,
		options: u32,
	) -> CFTypeRef;

	pub fn IOObjectRelease(obj: mach_port_t) -> kern_return_t;
}
