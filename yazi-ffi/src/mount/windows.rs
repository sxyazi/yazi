use std::{ffi::c_void, io, mem::size_of, ptr, sync::Arc};

use tokio::sync::Notify;
use windows::{Win32::Devices::DeviceAndDriverInstallation::HCMNOTIFICATION as Notification, core::Owned};
use windows_sys::Win32::{Devices::DeviceAndDriverInstallation::*, System::Ioctl::{GUID_DEVINTERFACE_DISK, GUID_DEVINTERFACE_PARTITION, GUID_DEVINTERFACE_VOLUME}};

use super::WindowMonitor;
use crate::device::config_ok;

pub struct Monitor {
	// Unregister callbacks before releasing their context in `window`.
	notifications: Vec<Owned<Notification>>,
	window:        WindowMonitor,
}

impl Monitor {
	pub fn new(notify: Arc<Notify>) -> io::Result<Self> {
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

		let mut monitor = Self { notifications: vec![], window: WindowMonitor::new(notify)? };
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
					Arc::as_ptr(&monitor.window.notify).cast(),
					Some(on_event),
					&mut handle,
				)
			})?;

			monitor.notifications.push(unsafe { Owned::new(Notification(handle)) });
		}

		Ok(monitor)
	}
}
