use std::{ffi::c_void, fs::OpenOptions, io, mem::size_of, os::windows::fs::OpenOptionsExt, ptr, sync::Arc};

use tokio::sync::Notify;
use windows::{Win32::Devices::DeviceAndDriverInstallation::HCMNOTIFICATION as Notification, core::Owned};
use windows_sys::Win32::{Devices::DeviceAndDriverInstallation::*, Foundation::GENERIC_READ, Storage::FileSystem::FILE_FLAG_OVERLAPPED, System::Ioctl::{GUID_DEVINTERFACE_DISK, GUID_DEVINTERFACE_PARTITION, GUID_DEVINTERFACE_VOLUME}};
use yazi_macro::error;

use super::DriveMonitor;
use crate::device::config_ok;

pub struct Monitor {
	// Unregister callbacks before releasing their context (`notify`).
	notifications: Vec<Owned<Notification>>,
	notify:        Arc<Notify>,
	drive_monitor: Option<DriveMonitor>,
}

// Registration handles can be unregistered from a different, non-callback thread.
// State access is synchronized; DriveMonitor drains callbacks before releasing it.
unsafe impl Send for Monitor {}

impl Monitor {
	pub async fn new(notify: Arc<Notify>) -> io::Result<Self> {
		tokio::task::spawn_blocking(move || Self::new_blocking(notify)).await?
	}

	fn new_blocking(notify: Arc<Notify>) -> io::Result<Self> {
		unsafe extern "system" fn on_event(
			_: HCMNOTIFICATION,
			context: *const c_void,
			_: CM_NOTIFY_ACTION,
			_: *const CM_NOTIFY_EVENT_DATA,
			_: u32,
		) -> u32 {
			unsafe { &*context.cast::<Notify>() }.notify_one();
			0
		}

		let mut monitor = Self { notifications: vec![], notify, drive_monitor: None };
		for class in [GUID_DEVINTERFACE_DISK, GUID_DEVINTERFACE_PARTITION, GUID_DEVINTERFACE_VOLUME] {
			let mut filter = CM_NOTIFY_FILTER {
				cbSize: size_of::<CM_NOTIFY_FILTER>() as _,
				FilterType: CM_NOTIFY_FILTER_TYPE_DEVICEINTERFACE,
				..Default::default()
			};
			filter.u.DeviceInterface.ClassGuid = class;

			let mut handle = ptr::null_mut();
			config_ok(unsafe {
				CM_Register_Notification(
					&filter,
					Arc::as_ptr(&monitor.notify).cast(),
					Some(on_event),
					&mut handle,
				)
			})?;

			monitor.notifications.push(unsafe { Owned::new(Notification(handle)) });
		}

		match OpenOptions::new()
			.access_mode(GENERIC_READ)
			.custom_flags(FILE_FLAG_OVERLAPPED)
			.open(r"\\.\MountPointManager")
		{
			Ok(device) => {
				monitor.drive_monitor = Some(DriveMonitor::new(device, monitor.notify.clone())?)
			}
			Err(e) => error!("Cannot monitor mount names: {e:?}"),
		};

		Ok(monitor)
	}
}
