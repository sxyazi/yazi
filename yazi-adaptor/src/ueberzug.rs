use std::{path::{Path, PathBuf}, process::Stdio};

use anyhow::{bail, Result};
use imagesize::ImageSize;
use ratatui::layout::Rect;
use tokio::{io::AsyncWriteExt, process::{Child, Command}, sync::mpsc::{self, UnboundedSender}};
use tracing::{debug, warn};
use yazi_config::PREVIEW;
use yazi_shared::RoCell;

use crate::{Adaptor, Image};

#[allow(clippy::type_complexity)]
static DEMON: RoCell<Option<UnboundedSender<Option<(PathBuf, Rect)>>>> = RoCell::new();

pub(super) struct Ueberzug;

impl Ueberzug {
	pub(super) fn start(adaptor: Adaptor) {
		if !adaptor.needs_ueberzug() {
			return DEMON.init(None);
		}

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
		DEMON.init(Some(tx))
	}

	pub(super) async fn image_show(path: &Path, rect: Rect) -> Result<(u32, u32)> {
		if let Some(tx) = &*DEMON {
			tx.send(Some((path.to_path_buf(), rect)))?;
			Adaptor::shown_store(rect, (0, 0));
		} else {
			bail!("uninitialized ueberzugpp");
		}

		let path = path.to_owned();
		let ImageSize { width: w, height: h } =
			tokio::task::spawn_blocking(move || imagesize::size(path)).await??;

		let (max_w, max_h) = Image::max_size(rect);
		if w <= max_w as usize && h <= max_h as usize {
			return Ok((w as u32, h as u32));
		}

		let ratio = f64::min(max_w as f64 / w as f64, max_h as f64 / h as f64);
		Ok(((w as f64 * ratio).round() as u32, (h as f64 * ratio).round() as u32))
	}

	pub(super) fn image_erase(_: Rect) -> Result<()> {
		if let Some(tx) = &*DEMON {
			Ok(tx.send(None)?)
		} else {
			bail!("uninitialized ueberzugpp");
		}
	}

	fn create_demon(adaptor: Adaptor) -> Result<Child> {
		// TODO: demon
		let result = Command::new("ueberzugpp")
			.args(["layer", "-so", &adaptor.to_string()])
			.kill_on_drop(true)
			.stdin(Stdio::piped())
			.stderr(Stdio::null())
			.spawn();

		if let Err(ref e) = result {
			warn!("ueberzugpp spawning failed: {e}");
		}
		Ok(result?)
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
			debug!("ueberzugpp rect before adjustment: {:?}", rect);
			let rect = Self::adjust_rect(rect);
			debug!("ueberzugpp rect after adjustment: {:?}", rect);

			let s = format!(
				r#"{{"action":"add","identifier":"yazi","x":{},"y":{},"max_width":{},"max_height":{},"path":"{}"}}{}"#,
				rect.x,
				rect.y,
				rect.width,
				rect.height,
				path.to_string_lossy(),
				"\n"
			);
			debug!("ueberzugpp command: {}", s);
			stdin.write_all(s.as_bytes()).await?;
		} else {
			debug!("ueberzugpp command: remove");
			stdin
				.write_all(format!(r#"{{"action":"remove","identifier":"yazi"}}{}"#, "\n").as_bytes())
				.await?;
		}
		Ok(())
	}
}
