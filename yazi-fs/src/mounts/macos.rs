use std::{ffi::{CStr, CString, OsString, c_void}, mem, os::unix::{ffi::OsStringExt, fs::MetadataExt}};

use anyhow::{Result, bail};
use core_foundation_sys::{array::CFArrayRef, base::{CFRelease, kCFAllocatorDefault}, runloop::{CFRunLoopGetCurrent, CFRunLoopRun, kCFRunLoopDefaultMode}};
use libc::{c_char, mach_port_t};
use objc::{msg_send, runtime::Object, sel, sel_impl};
use scopeguard::defer;
use tracing::error;
use yazi_ffi::{CFDict, CFString, DADiskCopyDescription, DADiskCreateFromBSDName, DARegisterDiskAppearedCallback, DARegisterDiskDescriptionChangedCallback, DARegisterDiskDisappearedCallback, DASessionCreate, DASessionScheduleWithRunLoop, IOIteratorNext, IOObjectRelease, IORegistryEntryCreateCFProperty, IOServiceGetMatchingServices, IOServiceMatching};
use yazi_shared::natsort;

use super::{Locked, Partition, Partitions};

impl Partitions {
	pub fn monitor<F>(me: Locked, cb: F)
	where
		F: Fn() + Copy + Send + 'static,
	{
		tokio::task::spawn_blocking(move || {
			let session = unsafe { DASessionCreate(kCFAllocatorDefault) };
			if session.is_null() {
				return error!("Cannot create a disk arbitration session");
			}
			defer! { unsafe { CFRelease(session) } };

			extern "C" fn on_appeared(_disk: *const c_void, context: *mut c_void) {
				let boxed = context as *mut Box<dyn Fn()>;
				unsafe { (*boxed)() }
			}

			extern "C" fn on_changed(_disk: *const c_void, _keys: CFArrayRef, context: *mut c_void) {
				let boxed = context as *mut Box<dyn Fn()>;
				unsafe { (*boxed)() }
			}

			extern "C" fn on_disappeared(_disk: *const c_void, context: *mut c_void) {
				let boxed = context as *mut Box<dyn Fn()>;
				unsafe { (*boxed)() }
			}

			let create_context = || {
				let me = me.clone();
				let boxed: Box<dyn Fn()> = Box::new(move || {
					if mem::replace(&mut me.write().need_update, true) {
						return;
					}
					Self::update(me.clone(), cb);
				});
				Box::into_raw(Box::new(boxed)) as *mut c_void
			};

			unsafe {
				DARegisterDiskAppearedCallback(session, std::ptr::null(), on_appeared, create_context());
				DARegisterDiskDescriptionChangedCallback(
					session,
					std::ptr::null(),
					std::ptr::null(),
					on_changed,
					create_context(),
				);
				DARegisterDiskDisappearedCallback(
					session,
					std::ptr::null(),
					on_disappeared,
					create_context(),
				);
				DASessionScheduleWithRunLoop(session, CFRunLoopGetCurrent(), kCFRunLoopDefaultMode);
				CFRunLoopRun();
			}
		});
	}

	fn update(me: Locked, cb: impl Fn() + Send + 'static) {
		_ = tokio::task::spawn_blocking(move || {
			let result = Self::all_names().and_then(Self::all_partitions);
			if let Err(ref e) = result {
				error!("Error encountered while updating mount points: {e:?}");
			}

			let mut guard = me.write();
			if let Ok(new) = result {
				guard.inner = new;
			}
			guard.need_update = false;

			drop(guard);
			cb();
		});
	}

	fn all_partitions(names: Vec<CString>) -> Result<Vec<Partition>> {
		let session = unsafe { DASessionCreate(kCFAllocatorDefault) };
		if session.is_null() {
			bail!("Cannot create a disk arbitration session");
		}
		defer! { unsafe { CFRelease(session) } };

		let mut disks = Vec::with_capacity(names.len());
		for name in names {
			let disk = unsafe { DADiskCreateFromBSDName(kCFAllocatorDefault, session, name.as_ptr()) };
			if disk.is_null() {
				continue;
			}

			defer! { unsafe { CFRelease(disk) } };
			let Ok(dict) = CFDict::take(unsafe { DADiskCopyDescription(disk) }) else {
				continue;
			};

			let partition = Partition::new(&OsString::from_vec(name.into_bytes()));
			let rdev = std::fs::metadata(&partition.src).map(|m| m.rdev() as _).ok();
			disks.push(Partition {
				dist: dict.path_buf("DAVolumePath").ok(),
				rdev,
				fstype: dict.os_string("DAVolumeKind").ok(),
				label: dict.os_string("DAVolumeName").ok(),
				capacity: dict.integer("DAMediaSize").unwrap_or_default() as u64,
				external: dict.bool("DADeviceInternal").ok().map(|b| !b),
				removable: dict.bool("DAMediaRemovable").ok(),
				..partition
			});
		}

		Ok(disks)
	}

	fn all_names() -> Result<Vec<CString>> {
		let mut iterator: mach_port_t = 0;
		let result = unsafe {
			IOServiceGetMatchingServices(0, IOServiceMatching(c"IOService".as_ptr()), &mut iterator)
		};

		if result != 0 {
			bail!("Cannot get the IO matching services");
		}
		defer! { unsafe { IOObjectRelease(iterator); } };

		let mut names = vec![];
		loop {
			let service = unsafe { IOIteratorNext(iterator) };
			if service == 0 {
				break;
			}
			defer! { unsafe { IOObjectRelease(service); } };
			if let Some(name) = Self::bsd_name(service).ok().filter(|s| s.as_bytes().starts_with(b"disk"))
			{
				names.push(name);
			}
		}

		names.sort_unstable_by(|a, b| natsort(a.as_bytes(), b.as_bytes(), false));
		Ok(names)
	}

	fn bsd_name(service: mach_port_t) -> Result<CString> {
		let key = CFString::new("BSD Name")?;
		let property =
			unsafe { IORegistryEntryCreateCFProperty(service, *key, kCFAllocatorDefault, 1) };
		if property.is_null() {
			bail!("Cannot get the name property");
		}
		defer! { unsafe { CFRelease(property) } };

		#[allow(unexpected_cfgs)]
		let cstr: *const c_char = unsafe { msg_send![property as *const Object, UTF8String] };
		Ok(if cstr.is_null() {
			bail!("Invalid value for the name property");
		} else {
			CString::from(unsafe { CStr::from_ptr(cstr) })
		})
	}
}
