use std::{ffi::c_void, fs::{File, FileTimes}, io, os::windows::io::BorrowedHandle, ptr};

use windows_sys::Win32::{Foundation::HANDLE, Storage::FileSystem::{CALLBACK_STREAM_SWITCH, CopyFileExW, PROGRESS_CONTINUE, PROGRESS_STOP}};
use yazi_shim::{ToWide, bool_ok};

use super::Copier;

impl Copier {
	pub(super) fn copy(&mut self) -> io::Result<()> {
		let from = self.from.to_wide_path()?;
		let to = self.to.to_wide_path()?;
		self.check()?;

		let mut context = Callback::new(self);
		let result = bool_ok(unsafe {
			CopyFileExW(
				from.as_ptr(),
				to.as_ptr(),
				Some(callback),
				(&raw mut context).cast(),
				ptr::null_mut(),
				0,
			)
		});

		context.error.map_or(result, Err)?;
		if let (Some(file), Some(times)) = (context.file, context.times) {
			file.set_times(times).ok();
		}

		Ok(())
	}

	pub(super) fn reset(&mut self) -> io::Result<()> { Ok(()) }

	pub(super) fn retry(&mut self, _error: &io::Error) -> bool { false }
}

// --- Callback
struct Callback<'a> {
	copier: &'a mut Copier,
	times:  Option<FileTimes>,
	file:   Option<File>,
	error:  Option<io::Error>,
}

impl<'a> Callback<'a> {
	fn new(copier: &'a mut Copier) -> Self {
		Self { times: copier.attrs.try_into().ok(), copier, file: None, error: None }
	}
}

unsafe extern "system" fn callback(
	_total: i64,
	_transferred: i64,
	_stream_size: i64,
	stream_bytes: i64,
	stream: u32,
	reason: u32,
	_source: HANDLE,
	destination: HANDLE,
	data: *const c_void,
) -> u32 {
	let context = unsafe { &mut *data.cast_mut().cast::<Callback<'_>>() };

	if let Err(e) = context.copier.check() {
		context.error = Some(e);
		return PROGRESS_STOP;
	}

	if stream == 1 && reason == CALLBACK_STREAM_SWITCH && context.times.is_some() {
		// Unlike std, we apply caller-supplied timestamps: duplicate the target handle so it survives CopyFileEx returning.
		context.file =
			unsafe { BorrowedHandle::borrow_raw(destination) }.try_clone_to_owned().ok().map(Into::into);
	}

	if stream == 1 {
		context.copier.progress.update(stream_bytes as u64);
		context.copier.progress.report(&context.copier.tx);
	}

	PROGRESS_CONTINUE
}
