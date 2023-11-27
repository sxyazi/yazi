use std::io::{stdout, Write};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use tokio::{io::{stdin, AsyncReadExt}, select, sync::mpsc, time};
use yazi_config::keymap::Exec;
use yazi_shared::{term::Term, Defer};

use crate::{emit, tasks::Tasks, Event, BLOCKER};

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}

impl Tasks {
	pub fn inspect(&self, _: impl Into<Opt>) -> bool {
		let Some(id) = self.scheduler.running.read().get_id(self.cursor) else {
			return false;
		};

		let scheduler = self.scheduler.clone();
		tokio::spawn(async move {
			let _guard = BLOCKER.acquire().await.unwrap();
			let (tx, mut rx) = mpsc::unbounded_channel();

			let buffered = {
				let mut running = scheduler.running.write();
				let Some(task) = running.get_mut(id) else { return };

				task.logger = Some(tx);
				task.logs.clone()
			};

			emit!(Stop(true)).await;
			let _defer = Defer::new(|| {
				disable_raw_mode().ok();
				Event::Stop(false, None).emit();
			});

			Term::clear(&mut stdout()).ok();
			stdout().write_all(buffered.as_bytes()).ok();
			enable_raw_mode().ok();

			let mut stdin = stdin();
			let mut quit = [0; 10];
			loop {
				select! {
					Some(line) = rx.recv() => {
						let mut stdout = stdout().lock();
						stdout.write_all(line.as_bytes()).ok();
						stdout.write_all(b"\r\n").ok();
					}
					_ = time::sleep(time::Duration::from_millis(100)) => {
						if scheduler.running.read().get(id).is_none() {
							stdout().write_all(b"Task finished, press `q` to quit\r\n").ok();
							break;
						}
					},
					Ok(_) = stdin.read(&mut quit) => {
						if quit[0] == b'q' {
							break;
						}
					}
				}
			}

			if let Some(task) = scheduler.running.write().get_mut(id) {
				task.logger = None;
			}
			while quit[0] != b'q' {
				stdin.read(&mut quit).await.ok();
			}
		});
		false
	}
}
