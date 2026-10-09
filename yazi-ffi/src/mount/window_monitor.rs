use std::{io, ptr, sync::Arc};

use scopeguard::{ScopeGuard, guard};
use tokio::sync::Notify;
use windows_sys::Win32::{Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM}, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::*};
use yazi_shim::{bool_ok, nz_ok};

pub(super) struct WindowMonitor {
	window:            HWND,
	instance:          HINSTANCE,
	pub(super) notify: Arc<Notify>,
}

impl WindowMonitor {
	const NAME: *const u16 = windows_sys::w!("YaziMountMonitor");

	pub fn new(notify: Arc<Notify>) -> io::Result<Self> {
		let instance = unsafe { GetModuleHandleW(ptr::null()) };
		bool_ok(!instance.is_null() as _)?;

		let class = WNDCLASSW {
			lpfnWndProc: Some(Self::on_event),
			hInstance: instance,
			lpszClassName: Self::NAME,
			..Default::default()
		};

		nz_ok(unsafe { RegisterClassW(&class) })?;
		let instance = guard(instance, |instance| unsafe {
			UnregisterClassW(Self::NAME, instance);
		});

		// Broadcasts do not reach message-only windows; keep an ordinary top-level window hidden.
		let window = unsafe {
			CreateWindowExW(
				0,
				Self::NAME,
				ptr::null(),
				0,
				0,
				0,
				0,
				0,
				ptr::null_mut(),
				ptr::null_mut(),
				*instance,
				Arc::as_ptr(&notify).cast(),
			)
		};
		bool_ok(!window.is_null() as _)?;

		Ok(Self { window, instance: ScopeGuard::into_inner(instance), notify })
	}

	unsafe extern "system" fn on_event(window: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
		unsafe {
			match msg {
				WM_NCCREATE => {
					let create = &*(l as *const CREATESTRUCTW);
					SetWindowLongPtrW(window, GWLP_USERDATA, create.lpCreateParams as _);
				}
				WM_DEVICECHANGE => {
					let notify = GetWindowLongPtrW(window, GWLP_USERDATA) as *const Notify;
					(*notify).notify_one();
				}
				// Keep teardown in Drop, together with the notification context.
				WM_CLOSE => return 0,
				_ => {}
			}
			DefWindowProcW(window, msg, w, l)
		}
	}
}

impl Drop for WindowMonitor {
	fn drop(&mut self) {
		unsafe {
			DestroyWindow(self.window);
			UnregisterClassW(Self::NAME, self.instance);
		}
	}
}
