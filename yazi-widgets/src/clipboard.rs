use parking_lot::Mutex;
use yazi_shim::cell::RoCell;

pub static CLIPBOARD: RoCell<Clipboard> = RoCell::new();

#[derive(Default)]
pub struct Clipboard {
	content: Mutex<Vec<u8>>,
}

pub struct ClipboardData {
	pub mime:    Vec<u8>,
	pub payload: Vec<u8>,
	pub alias:   Vec<u8>,
}

impl Clipboard {
	#[cfg(unix)]
	pub async fn get(&self) -> Vec<u8> {
		use tokio::process::Command;
		use yazi_shared::in_ssh_connection;

		if in_ssh_connection() {
			return self.content.lock().clone();
		}

		let all = [
			("pbpaste", &[][..]),
			("termux-clipboard-get", &[]),
			("wl-paste", &["-n"]),
			("xclip", &["-o", "-selection", "clipboard"]),
			("xsel", &["-ob"]),
		];

		for (bin, args) in all {
			let Ok(output) = Command::new(bin).args(args).kill_on_drop(true).output().await else {
				continue;
			};
			if output.status.success() {
				return output.stdout;
			}
		}
		self.content.lock().clone()
	}

	#[cfg(windows)]
	pub async fn get(&self) -> Vec<u8> {
		use clipboard_win::get_clipboard_string;

		let result = tokio::task::spawn_blocking(get_clipboard_string);
		if let Ok(Ok(s)) = result.await {
			return s.into_bytes();
		}

		self.content.lock().clone()
	}

	#[cfg(unix)]
	pub async fn set(&self, s: impl AsRef<[u8]>) {
		use std::process::Stdio;

		use tokio::{io::AsyncWriteExt, process::Command};
		use yazi_macro::writef;
		use yazi_tty::{TTY, sequence::SetClipboard};

		s.as_ref().clone_into(&mut self.content.lock());
		writef!(TTY.writer(), "{}", SetClipboard::new(s.as_ref())).ok();

		let all = [
			("pbcopy", &[][..]),
			("termux-clipboard-set", &[]),
			("wl-copy", &[]),
			("xclip", &["-selection", "clipboard"]),
			("xsel", &["-ib"]),
		];

		for (bin, args) in all {
			let cmd = Command::new(bin)
				.args(args)
				.stdin(Stdio::piped())
				.stdout(Stdio::null())
				.stderr(Stdio::null())
				.kill_on_drop(true)
				.spawn();

			let Ok(mut child) = cmd else { continue };

			let mut stdin = child.stdin.take().unwrap();
			if stdin.write_all(s.as_ref()).await.is_err() {
				continue;
			}
			drop(stdin);

			if child.wait().await.map(|s| s.success()).unwrap_or_default() {
				break;
			}
		}
	}

	#[cfg(windows)]
	pub async fn set(&self, s: impl AsRef<[u8]>) {
		use clipboard_win::set_clipboard_string;

		let b = s.as_ref().to_owned();
		*self.content.lock() = b.clone();

		tokio::task::spawn_blocking(move || set_clipboard_string(&String::from_utf8_lossy(&b)))
			.await
			.ok();
	}

	/// OSC 5522 Query MIME types
	pub async fn query_mime_types(&self) {
		use yazi_macro::writef;
		use yazi_tty::{TTY, sequence::ReadClipboardMimes};

		writef!(TTY.writer(), "{}", ReadClipboardMimes {}).ok();
	}

	/// OSC 5522 Clipboard read
	pub async fn read(&self, mime: impl AsRef<[u8]>, pw: impl AsRef<[u8]>) {
		use yazi_macro::writef;
		use yazi_tty::{TTY, sequence::ReadClipboard};

		writef!(TTY.writer(), "{}", ReadClipboard {
			mime:    mime.as_ref(),
			pw:      pw.as_ref(),
			name:    b"yazi",
			primary: false,
		})
		.ok();
	}

	/// OSC 5522 Clipboard write
	pub async fn write(&self, data: impl AsRef<[ClipboardData]>) {
		use yazi_macro::writef;
		use yazi_tty::{TTY, sequence::{WriteClipboard, WriteClipboardData}};

		let items = data
			.as_ref()
			.iter()
			.map(|d| WriteClipboardData {
				mime:    d.mime.as_ref(),
				payload: d.payload.as_ref(),
				alias:   d.alias.as_ref(),
			})
			.collect::<Vec<_>>();

		writef!(TTY.writer(), "{}", WriteClipboard { data: items }).ok();
	}
}
