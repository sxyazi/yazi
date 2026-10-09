use std::{future::{Future, poll_fn}, io, os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle}, pin::pin, ptr, sync::Arc, task::{Context, Poll, Wake, Waker}};

use windows_sys::Win32::{Foundation::{WAIT_FAILED, WAIT_OBJECT_0}, System::Threading::{CreateEventW, INFINITE, SetEvent}, UI::WindowsAndMessaging::*};
use yazi_shim::bool_ok;

pub struct MessageLoop(OwnedHandle);

impl MessageLoop {
	pub async fn run<F: Future>(future: F) -> io::Result<F::Output> {
		let handle = unsafe { CreateEventW(ptr::null(), 0, 0, ptr::null()) };
		bool_ok(!handle.is_null() as _)?;

		let waker = Waker::from(Arc::new(Self(unsafe { OwnedHandle::from_raw_handle(handle) })));
		let mut cx = Context::from_waker(&waker);

		let mut future = pin!(future);
		let mut msg = MSG::default();
		let mut wakes = 0;

		poll_fn(|outer| {
			if let Poll::Ready(result) = future.as_mut().poll(&mut cx) {
				return Ok(result).into();
			}

			// Check periodically even under continuous async wakes so window messages cannot starve.
			if wakes == 0 && unsafe { PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) } != 0 {
				if msg.message == WM_QUIT {
					return Err(io::ErrorKind::Interrupted.into()).into();
				}
				unsafe {
					TranslateMessage(&msg);
					DispatchMessageW(&msg);
				}
			}

			// Either an async wake or a window message must resume the main thread.
			match unsafe {
				MsgWaitForMultipleObjectsEx(1, &handle, INFINITE, QS_ALLINPUT, MWMO_INPUTAVAILABLE)
			} {
				WAIT_FAILED => return Err(io::Error::last_os_error()).into(),
				WAIT_OBJECT_0 => wakes = (wakes + 1) % 16,
				_ => wakes = 0,
			}

			// Resume Tokio after the Windows wait so each poll gets a fresh cooperative budget.
			outer.waker().wake_by_ref();
			Poll::Pending
		})
		.await
	}
}

impl Wake for MessageLoop {
	fn wake(self: Arc<Self>) { self.wake_by_ref(); }

	fn wake_by_ref(self: &Arc<Self>) {
		unsafe {
			SetEvent(self.0.as_raw_handle());
		}
	}
}
