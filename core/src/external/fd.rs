use std::{process::Stdio, time::Duration};

use anyhow::Result;
use shared::{StreamBuf, Url};
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command, sync::mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;

pub struct FdOpt {
	pub cwd:     Url,
	pub hidden:  bool,
	pub glob:    bool,
	pub subject: String,
}

pub fn fd(opt: FdOpt) -> Result<StreamBuf<UnboundedReceiverStream<Url>>> {
	let mut child = Command::new("fd")
		.arg("--base-directory")
		.arg(&opt.cwd)
		.arg(if opt.hidden { "--hidden" } else { "--no-hidden" })
		.arg(if opt.glob { "--glob" } else { "--regex" })
		.arg(&opt.subject)
		.kill_on_drop(true)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;

	drop(child.stderr.take());

	let mut it = BufReader::new(child.stdout.take().unwrap()).lines();
	let (tx, rx) = mpsc::unbounded_channel();
	let rx = StreamBuf::new(UnboundedReceiverStream::new(rx), Duration::from_millis(300));

	tokio::spawn(async move {
		while let Ok(Some(line)) = it.next_line().await {
			tx.send(opt.cwd.__join(line)).ok();
		}
		child.wait().await.ok();
	});
	Ok(rx)
}
