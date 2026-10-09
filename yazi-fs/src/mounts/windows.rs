use std::{ffi::OsString, io, os::windows::ffi::OsStringExt, path::PathBuf, ptr, sync::Arc};

use hashbrown::HashSet;
use scopeguard::defer;
use tokio::sync::Notify;
use windows_sys::Win32::{Foundation::{ERROR_MORE_DATA, ERROR_NO_MORE_FILES}, Storage::FileSystem::*, System::{Diagnostics::Debug::{SEM_FAILCRITICALERRORS, SetThreadErrorMode}, Ioctl::*, WindowsProgramming::{DRIVE_CDROM, DRIVE_FIXED, DRIVE_RAMDISK, DRIVE_REMOVABLE}}};
use yazi_ffi::{device, mount::Monitor};
use yazi_macro::{error, log_if_err};
use yazi_shim::{bool_ok, handle_ok};

use super::{Locked, Partition, Partitions};

impl Partitions {
	pub fn monitor<F>(me: &'static Locked, cb: F)
	where
		F: Fn() + 'static,
	{
		tokio::task::spawn_local(async move {
			let notify = Arc::new(Notify::new());
			let monitor = Monitor::new(notify.clone());
			log_if_err!("Monitoring partitions", &monitor);

			loop {
				match tokio::task::spawn_blocking(Self::all).await {
					Ok(Ok(new)) => me.write().inner = new,
					result => error!("Error encountered while updating partitions: {result:?}"),
				}

				cb();
				notify.notified().await;
			}
		});
	}

	fn all() -> io::Result<Vec<Partition>> {
		let mut mode = 0;
		bool_ok(unsafe { SetThreadErrorMode(SEM_FAILCRITICALERRORS, &mut mode) })?;
		defer! { unsafe { SetThreadErrorMode(mode, ptr::null_mut()); } };

		let mut result = vec![];
		let mut devices = HashSet::new();

		Self::volumes(&mut result, &mut devices)?;
		Self::disks(&mut result);
		Self::partitions(&mut result, devices);
		Self::drives(&mut result);

		result.sort_unstable_by(|a, b| (&a.src, &a.dist).cmp(&(&b.src, &b.dist)));
		Ok(result)
	}

	fn volumes(
		result: &mut Vec<Partition>,
		devices: &mut HashSet<(u32, u32, u32)>,
	) -> io::Result<()> {
		let mut name = [0; 50];
		let handle = match handle_ok(unsafe { FindFirstVolumeW(name.as_mut_ptr(), name.len() as _) }) {
			Err(e) if e.raw_os_error() == Some(ERROR_NO_MORE_FILES as _) => return Ok(()),
			Err(e) => return Err(e),
			Ok(handle) => handle,
		};

		defer! { unsafe { FindVolumeClose(handle); } };
		loop {
			let mut partition = Partition::volume(&name);
			devices.extend(device::partition_id(&partition.src));

			let mut paths = Self::volume_paths(&name).into_iter();
			let first = paths.next();
			result.extend(paths.map(|p| Partition { dist: Some(p), ..partition.clone() }));

			partition.dist = first;
			result.push(partition);

			match bool_ok(unsafe { FindNextVolumeW(handle, name.as_mut_ptr(), name.len() as _) }) {
				Ok(()) => {}
				Err(e) if e.raw_os_error() == Some(ERROR_NO_MORE_FILES as _) => break,
				Err(e) => return Err(e),
			}
		}

		Ok(())
	}

	fn volume_paths(name: &[u16]) -> Vec<PathBuf> {
		let mut buf = vec![0; 256];
		loop {
			let mut len = 0;
			let result = bool_ok(unsafe {
				GetVolumePathNamesForVolumeNameW(name.as_ptr(), buf.as_mut_ptr(), buf.len() as _, &mut len)
			});

			match result {
				Ok(()) => {
					return buf
						.split(|&c| c == 0)
						.take_while(|s| !s.is_empty())
						.map(|s| OsString::from_wide(s).into())
						.collect();
				}
				Err(e) if e.raw_os_error() == Some(ERROR_MORE_DATA as _) => buf.resize(len as _, 0),
				Err(_) => return vec![],
			}
		}
	}

	fn disks(result: &mut Vec<Partition>) {
		let disks = device::interfaces(&GUID_DEVINTERFACE_DISK);
		for src in disks.into_iter().flatten() {
			result.push(Partition { src, ..Default::default() });
		}
	}

	fn partitions(result: &mut Vec<Partition>, devices: HashSet<(u32, u32, u32)>) {
		let partitions = device::interfaces(&GUID_DEVINTERFACE_PARTITION);
		for src in partitions.into_iter().flatten() {
			if device::partition_id(&src).is_none_or(|id| !devices.contains(&id)) {
				result.push(Partition { src, ..Default::default() });
			}
		}
	}

	fn drives(result: &mut Vec<Partition>) {
		let drives = unsafe { GetLogicalDrives() };
		for i in 0..26 {
			if drives & (1 << i) == 0 {
				continue;
			}

			let root = [b'A' as u16 + i, b':' as _, b'\\' as _, 0];
			let dist: PathBuf = OsString::from_wide(&root[..3]).into();
			if result.iter().all(|p| p.dist.as_ref() != Some(&dist)) {
				result.push(Partition { dist: Some(dist), ..Partition::volume(&root) });
			}
		}
	}
}

impl Partition {
	fn volume(root: &[u16]) -> Self {
		let mut partition = Self {
			src: OsString::from_wide(root.split(|&c| c == 0).next().unwrap()),
			..Default::default()
		};

		// Label and fstype
		let mut label = [0; 261];
		let mut fstype = [0; 261];
		let result = unsafe {
			GetVolumeInformationW(
				root.as_ptr(),
				label.as_mut_ptr(),
				label.len() as _,
				ptr::null_mut(),
				ptr::null_mut(),
				ptr::null_mut(),
				fstype.as_mut_ptr(),
				fstype.len() as _,
			)
		};
		if bool_ok(result).is_ok() {
			partition.label = Some(OsString::from_wide(label.split(|&c| c == 0).next().unwrap()));
			partition.fstype = Some(OsString::from_wide(fstype.split(|&c| c == 0).next().unwrap()));
		}

		// Capacity
		unsafe {
			GetDiskFreeSpaceExW(root.as_ptr(), ptr::null_mut(), &mut partition.capacity, ptr::null_mut());
		}

		// Removable
		partition.removable = match unsafe { GetDriveTypeW(root.as_ptr()) } {
			DRIVE_REMOVABLE | DRIVE_CDROM => Some(true),
			DRIVE_FIXED | DRIVE_RAMDISK => Some(false),
			_ => None,
		};

		partition
	}
}
