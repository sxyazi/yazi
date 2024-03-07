use std::io::{stdout, Write};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use tokio::{io::{stdin, AsyncReadExt}, select, sync::mpsc, time};
use yazi_proxy::AppProxy;
use yazi_scheduler::HIDER;
use yazi_shared::{event::Cmd, term::Term, Defer};

use crate::tasks::Tasks;

impl Tasks {
	pub fn inspect(&self, _: Cmd) {
		let Some(id) = self.scheduler.ongoing.lock().get_id(self.cursor) else {
			return;
		};

		let scheduler = self.scheduler.clone();
		tokio::spawn(async move {
			let _permit = HIDER.acquire().await.unwrap();
			let (tx, mut rx) = mpsc::unbounded_channel();

			let buffered = {
				let mut ongoing = scheduler.ongoing.lock();
				let Some(task) = ongoing.get_mut(id) else { return };

				task.logger = Some(tx);
				task.logs.clone()
			};

			AppProxy::stop().await;
			let _defer = Defer::new(|| {
				disable_raw_mode().ok();
				AppProxy::resume();
			});

			Term::clear(&mut stdout()).ok();
			stdout().write_all(buffered.as_bytes()).ok();
			enable_raw_mode().ok();

			let mut stdin = stdin();
			let mut answer = 0;
			loop {
				select! {
					Some(line) = rx.recv() => {
						let mut stdout = stdout().lock();
						stdout.write_all(line.as_bytes()).ok();
						stdout.write_all(b"\r\n").ok();
					}
					_ = time::sleep(time::Duration::from_millis(500)) => {
						if scheduler.ongoing.lock().get(id).is_none() {
							stdout().write_all(b"Task finished, press `q` to quit\r\n").ok();
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

			if let Some(task) = scheduler.ongoing.lock().get_mut(id) {
				task.logger = None;
			}
			while answer != b'q' {
				answer = stdin.read_u8().await.unwrap_or(b'q');
			}
		});
	}
}
