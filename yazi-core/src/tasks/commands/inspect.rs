use std::io::{stdout, Write};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use tokio::{io::{stdin, AsyncReadExt}, select, sync::mpsc, time};
use yazi_scheduler::{Scheduler, BLOCKER};
use yazi_shared::{event::Cmd, term::Term, Defer};

use crate::tasks::Tasks;

impl Tasks {
	pub fn inspect(&self, _: Cmd) {
		let Some(id) = self.scheduler.running.lock().get_id(self.cursor) else {
			return;
		};

		let scheduler = self.scheduler.clone();
		tokio::spawn(async move {
			let _guard = BLOCKER.acquire().await.unwrap();
			let (tx, mut rx) = mpsc::unbounded_channel();

			let buffered = {
				let mut running = scheduler.running.lock();
				let Some(task) = running.get_mut(id) else { return };

				task.logger = Some(tx);
				task.logs.clone()
			};

			Scheduler::app_stop().await;
			let _defer = Defer::new(|| {
				disable_raw_mode().ok();
				Scheduler::app_resume();
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
						if scheduler.running.lock().get(id).is_none() {
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

			if let Some(task) = scheduler.running.lock().get_mut(id) {
				task.logger = None;
			}
			while answer != b'q' {
				answer = stdin.read_u8().await.unwrap_or(b'q');
			}
		});
	}
}
