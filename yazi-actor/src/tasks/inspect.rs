use std::io::Write;

use anyhow::Result;
use scopeguard::defer;
use tokio::{io::{AsyncReadExt, stdin}, select, sync::mpsc, time};
use yazi_binding::Permit;
use yazi_macro::{succ, writef};
use yazi_parser::VoidForm;
use yazi_scheduler::AppProxy;
use yazi_shared::data::Data;
use yazi_term::{TERM, YIELD_TO_SUBPROCESS, sequence::EraseScreen};
use yazi_tty::TTY;

use crate::{Actor, Ctx};

pub struct Inspect;

impl Actor for Inspect {
	type Form = VoidForm;

	const NAME: &str = "inspect";

	fn act(cx: &mut Ctx, _: Self::Form) -> Result<Data> {
		let ongoing = cx.tasks.scheduler.ongoing.clone();
		let Some(id) = ongoing.lock().get_id(cx.tasks.cursor) else {
			succ!();
		};

		tokio::spawn(async move {
			let _permit = Permit::new(YIELD_TO_SUBPROCESS.acquire().await.unwrap(), AppProxy::resume());
			let (tx, mut rx) = mpsc::unbounded_channel();

			let buffered = {
				let mut ongoing = ongoing.lock();
				let Some(task) = ongoing.get_mut(id) else { return };

				task.logger = Some(tx);
				task.logs.clone()
			};

			// Stop the app and clear the terminal
			AppProxy::stop().await;
			writeln!(TTY.writer(), "{EraseScreen}").ok();

			// Print the buffered logs
			TTY.writer().write_all(buffered.as_bytes()).ok();
			TTY.writer().flush().ok();

			defer! { TERM.enter_cooked_mode().ok(); }
			TERM.enter_raw_mode().ok();

			let mut stdin = stdin(); // TODO: stdin
			let mut answer = 0;
			loop {
				select! {
					Some(line) = rx.recv() => {
						writef!(TTY.writer(), "{line}\r\n").ok();
					}
					_ = time::sleep(time::Duration::from_millis(500)) => {
						if !ongoing.lock().exists(id) {
							writef!(TTY.writer(), "Task finished, press `q` to quit\r\n").ok();
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
		succ!();
	}
}
