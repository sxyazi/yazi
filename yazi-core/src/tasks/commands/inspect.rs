use std::io::Write;

use crossterm::{execute, terminal::{disable_raw_mode, enable_raw_mode}};
use scopeguard::defer;
use tokio::{io::{AsyncReadExt, stdin}, select, sync::mpsc, time};
use yazi_proxy::{AppProxy, HIDER};
use yazi_shared::{event::CmdCow, terminal_clear};
use yazi_term::tty::TTY;

use crate::tasks::Tasks;

impl Tasks {
	pub fn inspect(&self, _: CmdCow) {
		let ongoing = self.ongoing().clone();
		let Some(id) = ongoing.lock().get_id(self.cursor) else {
			return;
		};

		tokio::spawn(async move {
			let _permit = HIDER.acquire().await.unwrap();
			let (tx, mut rx) = mpsc::unbounded_channel();

			let buffered = {
				let mut ongoing = ongoing.lock();
				let Some(task) = ongoing.get_mut(id) else { return };

				task.logger = Some(tx);
				task.logs.clone()
			};

			defer!(AppProxy::resume());
			AppProxy::stop().await;

			terminal_clear(TTY.writer()).ok();
			TTY.writer().write_all(buffered.as_bytes()).ok();
			TTY.writer().flush().ok();

			defer! { disable_raw_mode().ok(); }
			enable_raw_mode().ok();

			let mut stdin = stdin(); // TODO: stdin
			let mut answer = 0;
			loop {
				select! {
					Some(line) = rx.recv() => {
						execute!(TTY.writer(), crossterm::style::Print(line), crossterm::style::Print("\r\n")).ok();
					}
					_ = time::sleep(time::Duration::from_millis(500)) => {
						if ongoing.lock().get(id).is_none() {
							execute!(TTY.writer(), crossterm::style::Print("Task finished, press `q` to quit\r\n")).ok();
							break;
						}
					},
					result = stdin.read_u8() => {
						answer = result.unwrap_or(b'q');
						if answer == b'q' {
							break;
						}
					}
				}
			}

			if let Some(task) = ongoing.lock().get_mut(id) {
				task.logger = None;
			}
			while answer != b'q' {
				answer = stdin.read_u8().await.unwrap_or(b'q');
			}
		});
	}
}
