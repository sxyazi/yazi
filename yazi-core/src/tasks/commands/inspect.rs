use std::{io::{stderr, BufWriter, LineWriter, Write}, mem};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use scopeguard::defer;
use tokio::{io::{stdin, AsyncReadExt}, select, sync::mpsc, time};
use yazi_proxy::{AppProxy, HIDER};
use yazi_shared::{event::Cmd, terminal_clear};

use crate::tasks::Tasks;

impl Tasks {
	pub fn inspect(&self, _: Cmd) {
		let ongoing = self.ongoing().clone();
		let Some(id) = ongoing.lock().get_id(self.cursor) else {
			return;
		};

		tokio::spawn(async move {
			let _permit = HIDER.acquire().await.unwrap();
			let (tx, mut rx) = mpsc::unbounded_channel();

			let mut buffered = {
				let mut ongoing = ongoing.lock();
				let Some(task) = ongoing.get_mut(id) else { return };

				task.logger = Some(tx);
				task.logs.clone()
			};

			defer!(AppProxy::resume());
			AppProxy::stop().await;

			terminal_clear(&mut stderr()).ok();
			BufWriter::new(stderr().lock()).write_all(mem::take(&mut buffered).as_bytes()).ok();

			defer! { disable_raw_mode().ok(); }
			enable_raw_mode().ok();

			let mut stdin = stdin();
			let mut answer = 0;
			loop {
				select! {
					Some(line) = rx.recv() => {
						let mut stderr = LineWriter::new(stderr().lock());
						stderr.write_all(line.as_bytes()).ok();
						stderr.write_all(b"\r\n").ok();
					}
					_ = time::sleep(time::Duration::from_millis(500)) => {
						if ongoing.lock().get(id).is_none() {
							stderr().write_all(b"Task finished, press `q` to quit\r\n").ok();
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
