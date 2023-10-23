use std::{cmp::max, path::PathBuf, process::Stdio};

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

	fn adjust_rect(mut rect: Rect) -> Rect {
		let cfg = &PREVIEW.ueberzug;
		rect.x = max((rect.x as f64 / cfg.scale_down_factor + cfg.x_offset) as i32, 0) as u16;
		rect.y = max((rect.y as f64 / cfg.scale_down_factor + cfg.y_offset) as i32, 0) as u16;
		rect.width =
			max((rect.width as f64 / cfg.scale_down_factor + cfg.width_offset) as i32, 0) as u16;
		rect.height =
			max((rect.height as f64 / cfg.scale_down_factor + cfg.height_offset) as i32, 0) as u16;
		return rect;
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

	async fn send_command(child: &mut Child, cmd: Option<(PathBuf, Rect)>) -> Result<()> {
		let stdin = child.stdin.as_mut().unwrap();
		if let Some((path, tmp_rect)) = cmd {
			debug!("ueberzug rect before adjustment: {:?}", tmp_rect);
			let rect = Self::adjust_rect(tmp_rect);
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
			stdin.write_all(s.as_bytes()).await?;
		} else {
			stdin
				.write_all(format!(r#"{{"action":"remove","identifier":"yazi"}}{}"#, "\n").as_bytes())
				.await?;
		}
		Ok(())
	}
}
