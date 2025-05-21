use std::ffi::{c_char, c_void};

use core_foundation_sys::{array::CFArrayRef, base::CFAllocatorRef, dictionary::CFDictionaryRef, runloop::CFRunLoopRef, string::CFStringRef};

#[link(name = "DiskArbitration", kind = "framework")]
unsafe extern "C" {
	pub fn DASessionCreate(allocator: CFAllocatorRef) -> *const c_void;

	pub fn DADiskCreateFromBSDName(
		allocator: CFAllocatorRef,
		session: *const c_void,
		path: *const c_char,
	) -> *const c_void;

	pub fn DADiskCopyDescription(disk: *const c_void) -> CFDictionaryRef;

	pub fn DARegisterDiskAppearedCallback(
		session: *const c_void,
		r#match: CFDictionaryRef,
		callback: extern "C" fn(disk: *const c_void, context: *mut c_void),
		context: *mut c_void,
	);

	pub fn DARegisterDiskDescriptionChangedCallback(
		session: *const c_void,
		r#match: CFDictionaryRef,
		watch: CFArrayRef,
		callback: extern "C" fn(disk: *const c_void, keys: CFArrayRef, context: *mut c_void),
		context: *mut c_void,
	);

	pub fn DARegisterDiskDisappearedCallback(
		session: *const c_void,
		r#match: CFDictionaryRef,
		callback: extern "C" fn(disk: *const c_void, context: *mut c_void),
		context: *mut c_void,
	);

	pub fn DASessionScheduleWithRunLoop(
		session: *const c_void,
		runLoop: CFRunLoopRef,
		runLoopMode: CFStringRef,
	);
}
