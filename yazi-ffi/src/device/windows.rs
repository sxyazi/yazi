use std::{ffi::{OsStr, OsString}, fs::OpenOptions, io, mem::size_of_val, os::windows::{ffi::OsStringExt, fs::OpenOptionsExt, io::AsRawHandle}, ptr};

use windows_sys::{Win32::{Devices::DeviceAndDriverInstallation::*, Foundation::ERROR_GEN_FAILURE, System::{IO::DeviceIoControl, Ioctl::{IOCTL_STORAGE_GET_DEVICE_NUMBER, STORAGE_DEVICE_NUMBER}}}, core::GUID};
use yazi_shim::{bool_ok, wtf8::FromWtf8};

pub(crate) fn config_ok(r: CONFIGRET) -> io::Result<()> {
	if r == CR_SUCCESS {
		Ok(())
	} else {
		Err(io::Error::from_raw_os_error(unsafe { CM_MapCrToWin32Err(r, ERROR_GEN_FAILURE) } as _))
	}
}

pub fn interfaces(class: &GUID) -> io::Result<Vec<OsString>> {
	loop {
		let mut len = 0;
		config_ok(unsafe {
			CM_Get_Device_Interface_List_SizeW(
				&mut len,
				class,
				ptr::null(),
				CM_GET_DEVICE_INTERFACE_LIST_PRESENT,
			)
		})?;

		let mut buf = vec![0; len as usize];
		let result = unsafe {
			CM_Get_Device_Interface_ListW(
				class,
				ptr::null(),
				buf.as_mut_ptr(),
				len,
				CM_GET_DEVICE_INTERFACE_LIST_PRESENT,
			)
		};
		if result == CR_BUFFER_SMALL {
			continue;
		}

		config_ok(result)?;
		return Ok(
			buf.split(|&c| c == 0).take_while(|b| !b.is_empty()).map(OsString::from_wide).collect(),
		);
	}
}

pub fn partition_id(src: &OsStr) -> Option<(u32, u32, u32)> {
	let name = src.as_encoded_bytes();
	let name = OsStr::from_wtf8(name.strip_suffix(b"\\").unwrap_or(name)).ok()?;
	let device = OpenOptions::new().access_mode(0).open(name).ok()?;

	let mut number: STORAGE_DEVICE_NUMBER = Default::default();
	let mut len = 0;
	bool_ok(unsafe {
		DeviceIoControl(
			device.as_raw_handle(),
			IOCTL_STORAGE_GET_DEVICE_NUMBER,
			ptr::null(),
			0,
			(&raw mut number).cast(),
			size_of_val(&number) as _,
			&mut len,
			ptr::null_mut(),
		)
	})
	.ok()?;

	match number.PartitionNumber {
		_ if len as usize != size_of_val(&number) => None,
		_ if number.DeviceNumber == u32::MAX => None,
		n if n == 0 || n == u32::MAX => None,
		n => Some((number.DeviceType, number.DeviceNumber, n)),
	}
}
