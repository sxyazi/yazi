use std::{path::PathBuf, process::Stdio};

use anyhow::Result;
use ratatui::prelude::Rect;
use tokio::{io::AsyncWriteExt, process::{Child, Command}, sync::mpsc::{self, UnboundedSender}};
use tracing::debug;
use yazi_config::PREVIEW;

use crate::Adaptor;

pub(super) struct Ueberzug;

impl Ueberzug {
	pub(super) fn start(adaptor: Adaptor) -> Result<UnboundedSender<Option<(PathBuf, Rect)>>> {
		let mut child = Self::create_demon(adaptor).ok();
		let (tx, mut rx) = mpsc::unbounded_channel();

		tokio::spawn(async move {
			while let Some(cmd) = rx.recv().await {
				let exit = child.as_mut().and_then(|c| c.try_wait().ok());
				if exit != Some(None) {
					child = None;
				}
				if child.is_none() {
					child = Self::create_demon(adaptor).ok();
				}
				if let Some(c) = &mut child {
					Self::send_command(c, cmd).await.ok();
				}
			}
		});

		Ok(tx)
	}

	fn create_demon(adaptor: Adaptor) -> Result<Child> {
		Ok(
			Command::new("ueberzug")
				.args(["layer", "-so", &adaptor.to_string()])
				.kill_on_drop(true)
				.stdin(Stdio::piped())
				.stderr(Stdio::null())
				.spawn()?,
		)
	}

	fn adjust_rect(mut rect: Rect) -> Rect {
		let scale = PREVIEW.ueberzug_scale;
		let (x, y, w, h) = PREVIEW.ueberzug_offset;

		rect.x = 0f32.max(rect.x as f32 * scale + x) as u16;
		rect.y = 0f32.max(rect.y as f32 * scale + y) as u16;
		rect.width = 0f32.max(rect.width as f32 * scale + w) as u16;
		rect.height = 0f32.max(rect.height as f32 * scale + h) as u16;
		rect
	}

	async fn send_command(child: &mut Child, cmd: Option<(PathBuf, Rect)>) -> Result<()> {
		let stdin = child.stdin.as_mut().unwrap();
		if let Some((path, rect)) = cmd {
			debug!("ueberzug rect before adjustment: {:?}", rect);
			let rect = Self::adjust_rect(rect);
			debug!("ueberzug rect after adjustment: {:?}", rect);

			let s = format!(
				r#"{{"action":"add","identifier":"yazi","x":{},"y":{},"max_width":{},"max_height":{},"path":"{}"}}{}"#,
				rect.x,
				rect.y,
				rect.width,
				rect.height,
				path.to_string_lossy(),
				"\n"
			);
			debug!("ueberzug command: {}", s);
			stdin.write_all(s.as_bytes()).await?;
		} else {
			debug!("ueberzug command: remove");
			stdin
				.write_all(format!(r#"{{"action":"remove","identifier":"yazi"}}{}"#, "\n").as_bytes())
				.await?;
		}
		Ok(())
	}
}
