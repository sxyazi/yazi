use std::{ffi::{CString, OsStr, c_void}, mem, os::unix::{ffi::OsStrExt, fs::MetadataExt}, ptr::NonNull};

use anyhow::{Context, Result, bail};
use dispatch2::{DispatchObject, DispatchQueue};
use objc2_core_foundation::{CFArray, CFBoolean, CFDictionary, CFNumber, CFRetained, CFString, CFType, CFURL, ConcreteType};
use objc2_disk_arbitration::{DADisk, DARegisterDiskAppearedCallback, DARegisterDiskDescriptionChangedCallback, DARegisterDiskDisappearedCallback, DASession, kDADiskDescriptionDeviceInternalKey, kDADiskDescriptionMediaRemovableKey, kDADiskDescriptionMediaSizeKey, kDADiskDescriptionVolumeKindKey, kDADiskDescriptionVolumeNameKey, kDADiskDescriptionVolumePathKey};
use scopeguard::defer;
use yazi_ffi::{IOIteratorNext, IOObjectRelease, IORegistryEntryCreateCFProperty, IOServiceGetMatchingServices, IOServiceMatching};
use yazi_macro::error;
use yazi_shared::natsort;

use super::{Locked, Partition, Partitions};

impl Partitions {
	pub fn monitor<F>(me: &'static Locked, cb: F)
	where
		F: Fn() + Copy + Send + 'static,
	{
		let rt = tokio::runtime::Handle::current();
		let Some(session) = (unsafe { DASession::new(None) }) else {
			return error!("Cannot create a disk arbitration session");
		};

		extern "C-unwind" fn on_event(_disk: NonNull<DADisk>, context: *mut c_void) {
			let boxed = context as *mut Box<dyn Fn() + Send>;
			unsafe { (*boxed)() }
		}

		extern "C-unwind" fn on_changed(
			disk: NonNull<DADisk>,
			_keys: NonNull<CFArray>,
			context: *mut c_void,
		) {
			on_event(disk, context);
		}

		let callback: Box<dyn Fn() + Send> = Box::new(move || Self::update(me, cb, &rt));
		let mut callback = Box::new(callback);

		let context = (&raw mut *callback).cast();

		let queue = DispatchQueue::new("yazi.mounts", None);
		queue.set_finalizer(move || drop(callback));

		unsafe {
			DARegisterDiskAppearedCallback(&session, None, Some(on_event), context);
			DARegisterDiskDescriptionChangedCallback(&session, None, None, Some(on_changed), context);
			DARegisterDiskDisappearedCallback(&session, None, Some(on_event), context);
			// The dispatch source retains the session and queue, which owns the callback.
			session.set_dispatch_queue(Some(&queue));
		}
	}

	fn update<F>(me: &'static Locked, cb: F, rt: &tokio::runtime::Handle)
	where
		F: Fn() + Send + 'static,
	{
		if mem::replace(&mut me.write().need_update, true) {
			return;
		}

		_ = rt.spawn_blocking(move || {
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
		fn value<T: ConcreteType>(
			dict: &CFDictionary<CFString, CFType>,
			key: &CFString,
		) -> Option<CFRetained<T>> {
			dict.get(key)?.downcast().ok()
		}

		let session =
			unsafe { DASession::new(None) }.context("Cannot create a disk arbitration session")?;

		let mut disks = Vec::with_capacity(names.len());
		for name in names {
			let Some(disk) =
				(unsafe { DADisk::from_bsd_name(None, &session, NonNull::from(&*name).cast()) })
			else {
				continue;
			};

			// Disk descriptions have `CFString` keys and `CFType` values.
			let Some(dict) = (unsafe { disk.description() }) else { continue };
			let dict = unsafe { CFRetained::cast_unchecked(dict) };

			let partition = Partition::new(OsStr::from_bytes(name.as_bytes()));
			let rdev = std::fs::metadata(&partition.src).map(|m| m.rdev() as _).ok();
			unsafe {
				disks.push(Partition {
					dist: value::<CFURL>(&dict, kDADiskDescriptionVolumePathKey)
						.and_then(|v| v.to_file_path()),
					rdev,
					fstype: value::<CFString>(&dict, kDADiskDescriptionVolumeKindKey)
						.map(|v| v.to_string().into()),
					label: value::<CFString>(&dict, kDADiskDescriptionVolumeNameKey)
						.map(|v| v.to_string().into()),
					capacity: value::<CFNumber>(&dict, kDADiskDescriptionMediaSizeKey)
						.and_then(|v| v.as_i64())
						.unwrap_or_default() as u64,
					external: value::<CFBoolean>(&dict, kDADiskDescriptionDeviceInternalKey)
						.map(|v| !v.as_bool()),
					removable: value::<CFBoolean>(&dict, kDADiskDescriptionMediaRemovableKey)
						.map(|v| v.as_bool()),
					..partition
				});
			}
		}

		Ok(disks)
	}

	fn all_names() -> Result<Vec<CString>> {
		let mut iterator = 0;
		let result = unsafe {
			IOServiceGetMatchingServices(0, IOServiceMatching(c"IOMedia".as_ptr()), &mut iterator)
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

	fn bsd_name(service: u32) -> Result<CString> {
		let key = CFString::from_static_str("BSD Name");

		let prop = unsafe { IORegistryEntryCreateCFProperty(service, &key, None, 1) }
			.context("Cannot get the name property")?;
		let prop = unsafe { CFRetained::from_raw(prop) };
		let prop: CFRetained<CFString> =
			prop.downcast().ok().context("Invalid value for the name property")?;

		Ok(CString::new(prop.to_string())?)
	}
}
