use tokio_util::sync::CancellationToken;
use yazi_adaptor::ADAPTOR;
use yazi_config::PLUGIN;
use yazi_plugin::{external::Highlighter, utils::PreviewLock};
use yazi_shared::fs::{Cha, File, Url};

#[derive(Default)]
pub struct Preview {
	pub lock: Option<PreviewLock>,
	pub skip: usize,

	previewer_ct: Option<CancellationToken>,
}

impl Preview {
	pub fn go(&mut self, file: File, mime: String) {
		if !self.content_unchanged(&file.url, &file.cha) {
			self.force(file, mime);
		}
	}

	pub fn force(&mut self, file: File, mime: String) {
		let Some(previewer) = PLUGIN.previewer(&file.url, &mime) else {
			self.reset();
			return;
		};

		self.abort();
		if previewer.sync {
			yazi_plugin::isolate::peek_sync(&previewer.exec, file, self.skip);
		} else {
			self.previewer_ct = Some(yazi_plugin::isolate::peek(&previewer.exec, file, self.skip));
		}
	}

	#[inline]
	pub fn abort(&mut self) {
		self.previewer_ct.take().map(|ct| ct.cancel());
		Highlighter::abort();
	}

	#[inline]
	pub fn reset(&mut self) -> bool {
		self.abort();
		ADAPTOR.image_hide().ok();
		self.lock.take().is_some()
	}

	#[inline]
	pub fn reset_image(&mut self) {
		self.abort();
		ADAPTOR.image_hide().ok();
	}

	#[inline]
	pub fn same_url(&self, url: &Url) -> bool {
		matches!(self.lock, Some(ref lock) if lock.url == *url)
	}

	fn content_unchanged(&self, url: &Url, cha: &Cha) -> bool {
		let Some(lock) = &self.lock else {
			return false;
		};

		*url == lock.url
			&& self.skip == lock.skip
			&& cha.len == lock.cha.len
			&& cha.modified == lock.cha.modified
			&& cha.kind == lock.cha.kind
			&& {
				#[cfg(unix)]
				{
					cha.permissions == lock.cha.permissions
				}
				#[cfg(windows)]
				{
					true
				}
			}
	}
}
