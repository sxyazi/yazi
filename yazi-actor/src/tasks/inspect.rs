use std::io::Write;

use anyhow::Result;
use crossterm::{execute, terminal::{disable_raw_mode, enable_raw_mode}};
use scopeguard::defer;
use tokio::{io::{AsyncReadExt, stdin}, select, sync::mpsc, time};
use yazi_binding::Permit;
use yazi_macro::succ;
use yazi_parser::VoidOpt;
use yazi_proxy::{AppProxy, HIDER};
use yazi_shared::{data::Data, terminal_clear};
use yazi_term::tty::TTY;

use crate::{Actor, Ctx};

pub struct Inspect;

impl Actor for Inspect {
	type Options = VoidOpt;

	const NAME: &str = "inspect";

	fn act(cx: &mut Ctx, _: Self::Options) -> Result<Data> {
		let ongoing = cx.tasks.ongoing().clone();
		let Some(id) = ongoing.lock().get_id(cx.tasks.cursor) else {
			succ!();
		};

		tokio::spawn(async move {
			let _permit = Permit::new(HIDER.acquire().await.unwrap(), AppProxy::resume());
			let (tx, mut rx) = mpsc::unbounded_channel();

			let buffered = {
				let mut ongoing = ongoing.lock();
				let Some(task) = ongoing.get_mut(id) else { return };

				task.logger = Some(tx);
				task.logs.clone()
			};

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
						if !ongoing.lock().exists(id) {
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
		succ!();
	}
}
