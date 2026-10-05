use std::{cell::UnsafeCell, ffi::c_void, fs::File, io, mem::size_of, os::windows::io::AsRawHandle, ptr, sync::Arc};

use parking_lot::Mutex;
use tokio::sync::Notify;
use windows::{Win32::System::Threading::PTP_IO as Pool, core::Owned};
use windows_sys::Win32::{Foundation::ERROR_IO_PENDING, System::{IO::{CancelIoEx, DeviceIoControl, OVERLAPPED}, Threading::{CancelThreadpoolIo, CreateThreadpoolIo, PTP_CALLBACK_INSTANCE, PTP_IO, StartThreadpoolIo, WaitForThreadpoolIoCallbacks}}};
use yazi_macro::log_if_err;
use yazi_shim::{bool_ok, nz_ok};

pub(super) struct DriveMonitor {
	// Drop the state (and file) before the threadpool handle.
	state: Arc<State>,
	pool:  Owned<Pool>,
}

impl Drop for DriveMonitor {
	fn drop(&mut self) {
		*self.state.closed.lock() = true;

		unsafe {
			CancelIoEx(self.state.device.as_raw_handle(), ptr::null());
			// Keep completion callbacks enabled so canceled I/O is fully drained.
			WaitForThreadpoolIoCallbacks(self.pool.0, 0);
		}
	}
}

impl DriveMonitor {
	pub(super) fn new(device: File, notify: Arc<Notify>) -> io::Result<Self> {
		unsafe extern "system" fn on_event(
			_: PTP_CALLBACK_INSTANCE,
			context: *mut c_void,
			_: *mut c_void,
			result: u32,
			len: usize,
			pool: PTP_IO,
		) {
			let me = unsafe { &*context.cast::<State>() };

			let closed = me.closed.lock();
			if *closed {
				return;
			}

			let result = if result != 0 {
				Err(io::Error::from_raw_os_error(result as _))
			} else if len != size_of::<u32>() {
				Err(io::ErrorKind::UnexpectedEof.into())
			} else {
				me.notify.notify_one();
				unsafe { me.submit(pool) }
			};
			log_if_err!("Monitoring mount names", result);
		}

		#[expect(
			clippy::arc_with_non_send_sync,
			reason = "Monitor owns synchronization and callback lifetime"
		)]
		let state = Arc::new(State {
			device,
			notify,
			closed: Mutex::new(false),
			overlapped: UnsafeCell::new(Default::default()),
			epic: UnsafeCell::new(0),
		});

		let pool = nz_ok(unsafe {
			CreateThreadpoolIo(
				state.device.as_raw_handle(),
				Some(on_event),
				Arc::as_ptr(&state).cast_mut().cast(),
				ptr::null(),
			)
		})?;

		let me = Self { state, pool: unsafe { Owned::new(Pool(pool)) } };
		{
			let _closed = me.state.closed.lock();
			log_if_err!("Monitoring mount names", unsafe { me.state.submit(pool) });
		}
		Ok(me)
	}
}

// --- State
struct State {
	device:     File,
	notify:     Arc<Notify>,
	closed:     Mutex<bool>,
	overlapped: UnsafeCell<OVERLAPPED>,
	epic:       UnsafeCell<u32>,
}

impl State {
	// The caller holds `closed`; the previous request has completed, if any.
	unsafe fn submit(&self, pool: PTP_IO) -> io::Result<()> {
		let result = unsafe {
			self.overlapped.get().write(Default::default());
			StartThreadpoolIo(pool);
			DeviceIoControl(
				self.device.as_raw_handle(),
				0x6d4020, // IOCTL_MOUNTMGR_CHANGE_NOTIFY
				self.epic.get().cast(),
				size_of::<u32>() as _,
				self.epic.get().cast(),
				size_of::<u32>() as _,
				ptr::null_mut(),
				self.overlapped.get(),
			)
		};

		match bool_ok(result) {
			Ok(()) => Ok(()),
			Err(e) if e.raw_os_error() == Some(ERROR_IO_PENDING as _) => Ok(()),
			Err(e) => {
				unsafe { CancelThreadpoolIo(pool) };
				Err(e)
			}
		}
	}
}
