use std::{path::PathBuf, process::Stdio};

use anyhow::{Ok, Result};
use ratatui::prelude::Rect;
use tokio::{io::AsyncWriteExt, process::{Child, Command}, select, sync::mpsc::{self, UnboundedSender}};

pub struct Ueberzug;

impl Ueberzug {
	pub fn init() -> Result<UnboundedSender<Option<(PathBuf, Rect)>>> {
		let mut child = Some(Self::create_demon()?);
		let (tx, mut rx) = mpsc::unbounded_channel();

		tokio::spawn(async move {
			loop {
				if let Some(c) = &mut child {
					select! {
						_ = c.wait() => child = None,
						data = rx.recv() => {
							if let Some(img) = data {
								Self::send_command(c, img).await.ok();
							} else {
								break;
							}
						},
					}
				} else if let Some(img) = rx.recv().await {
					child = Self::create_demon().ok();
					if let Some(c) = &mut child {
						Self::send_command(c, img).await.ok();
					}
				} else {
					break;
				}
			}
		});

		Ok(tx)
	}

	fn create_demon() -> Result<Child> {
		Ok(
			Command::new("ueberzug")
			.args(["layer", "-s", "--use-escape-codes"])
			.kill_on_drop(true)
			.stdin(Stdio::piped())
			// .stdout(Stdio::piped())
			.stderr(Stdio::null())
			.spawn()?,
		)
	}

	async fn send_command(child: &mut Child, img: Option<(PathBuf, Rect)>) -> Result<()> {
		let stdin = child.stdin.as_mut().unwrap();
		if let Some((path, rect)) = img {
			stdin.write_all(br#"{"action":"add","identifier":"preview","max_height":0,"max_width":0,"path":"/Users/ika/Downloads/photo_2023-06-28 07.23.33.jpeg","x":0,"y":0}\n"#).await?;
		} else {
			stdin.write_all(br#"{"action":"remove","identifier":"preview"}\n"#).await?;
		}
		Ok(())
	}
}
