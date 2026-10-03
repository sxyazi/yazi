use std::{mem::MaybeUninit, ops::Deref, ptr, ptr::NonNull, sync::LazyLock};

use chrono::{DateTime, Local, Utc};

use super::Locale;

unsafe extern "C" {
	#[cfg(unix)]
	fn wcsftime_l(
		output: *mut libc::wchar_t,
		size: libc::size_t,
		format: *const libc::wchar_t,
		time: *const libc::tm,
		locale: libc::locale_t,
	) -> libc::size_t;

	#[cfg(windows)]
	fn _create_locale(category: libc::c_int, locale: *const libc::c_char) -> *mut libc::c_void;

	#[cfg(windows)]
	fn _wcsftime_l(
		output: *mut libc::wchar_t,
		size: libc::size_t,
		format: *const libc::wchar_t,
		time: *const libc::tm,
		locale: *mut libc::c_void,
	) -> libc::size_t;
}

impl Locale {
	pub fn strftime(date: DateTime<Utc>) -> String {
		Self::strftime_imp(date).unwrap_or_else(|| date.with_timezone(&Local).format("%c").to_string())
	}

	#[cfg(unix)]
	fn strftime_imp(date: DateTime<Utc>) -> Option<String> {
		let secs: libc::time_t = date.timestamp().try_into().ok()?;
		let mut tm = MaybeUninit::uninit();
		if unsafe { libc::localtime_r(&secs, tm.as_mut_ptr()) }.is_null() {
			return None;
		}

		let mut buf = [0 as libc::wchar_t; 250];
		let len = unsafe {
			wcsftime_l(
				buf.as_mut_ptr(),
				buf.len(),
				[b'%' as libc::wchar_t, b'c' as libc::wchar_t, 0].as_ptr(),
				tm.as_ptr(),
				Lc::get()?.as_ptr().cast(),
			)
		};
		if len == 0 {
			return None;
		}

		buf[..len].iter().map(|&c| char::from_u32(c as u32)).collect()
	}

	#[cfg(windows)]
	fn strftime_imp(date: DateTime<Utc>) -> Option<String> {
		let secs: libc::time_t = date.timestamp().try_into().ok()?;
		let mut tm = MaybeUninit::uninit();
		if unsafe { libc::localtime_s(tm.as_mut_ptr(), &secs) } != 0 {
			return None;
		}

		let mut buf = [0 as libc::wchar_t; 250];
		let len = unsafe {
			_wcsftime_l(
				buf.as_mut_ptr(),
				buf.len(),
				[b'%' as libc::wchar_t, b'c' as libc::wchar_t, 0].as_ptr(),
				tm.as_ptr(),
				Lc::get()?.as_ptr(),
			)
		};
		if len == 0 {
			return None;
		}

		String::from_utf16(&buf[..len]).ok()
	}
}

// --- Lc
struct Lc(NonNull<libc::c_void>);

// The cached locale is immutable, shared only by formatting calls, and never freed.
unsafe impl Send for Lc {}
unsafe impl Sync for Lc {}

impl Deref for Lc {
	type Target = NonNull<libc::c_void>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Lc {
	pub fn get() -> Option<&'static Self> {
		static CACHE: LazyLock<Option<Lc>> = LazyLock::new(|| unsafe {
			let name = libc::setlocale(libc::LC_TIME, ptr::null());
			if name.is_null() {
				return None;
			}

			#[cfg(unix)]
			let locale = libc::newlocale(libc::LC_TIME_MASK | libc::LC_CTYPE_MASK, name, ptr::null_mut());
			#[cfg(windows)]
			let locale = _create_locale(libc::LC_ALL, name);

			NonNull::new(locale.cast()).map(Lc)
		});

		CACHE.as_ref()
	}
}
