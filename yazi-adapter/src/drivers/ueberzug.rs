use std::{path::{Path, PathBuf}, process::Stdio};

use anyhow::{Result, bail};
use ratatui::layout::{Rect, Size};
use tokio::{io::AsyncWriteExt, process::{Child, Command}, sync::mpsc::{self, UnboundedSender}};
use tracing::{debug, warn};
use yazi_config::PREVIEW;
use yazi_shared::{RoCell, env_exists};

use crate::{Adapter, Dimension, Offset};

type Cmd = Option<(PathBuf, Rect)>;

static DEMON: RoCell<Option<UnboundedSender<Cmd>>> = RoCell::new();

pub(crate) struct Ueberzug;

impl Ueberzug {
	pub(crate) fn start(adapter: Adapter) {
		if !adapter.needs_ueberzug() {
			return DEMON.init(None);
		}

		let mut child = Self::create_demon(adapter).ok();
		let (tx, mut rx) = mpsc::unbounded_channel();

		tokio::spawn(async move {
			while let Some(cmd) = rx.recv().await {
				let exit = child.as_mut().and_then(|c| c.try_wait().ok());
				if exit != Some(None) {
					child = None;
				}
				if child.is_none() {
					child = Self::create_demon(adapter).ok();
				}
				if let Some(c) = &mut child {
					Self::send_command(adapter, c, cmd).await.ok();
				}
			}
		});
		DEMON.init(Some(tx))
	}

	pub(crate) async fn image_show(path: &Path, max: Rect, offset: Option<Offset>) -> Result<Rect> {
		let Some(tx) = &*DEMON else {
			bail!("uninitialized ueberzugpp");
		};

		let p = path.to_owned();
		let (w, h) = tokio::task::spawn_blocking(move || image::image_dimensions(p)).await??;

		let area = Dimension::ratio()
			.map(|(r1, r2)| {
				let width = max.width.min((w.min(PREVIEW.max_width as _) as f64 / r1).ceil() as _);
				let height = max.height.min((h.min(PREVIEW.max_height as _) as f64 / r2).ceil() as _);
				let offset = offset.unwrap_or_else(|| {
					Offset::from((Size { width, height }, Size { width: max.width, height: max.height }))
				});
				Rect { x: max.x + offset.x, y: max.y + offset.y, width, height }
			})
			.unwrap_or(max);

		tx.send(Some((path.to_owned(), area)))?;
		Adapter::shown_store(area);
		Ok(area)
	}

	pub(crate) fn image_erase(_: Rect) -> Result<()> {
		if let Some(tx) = &*DEMON {
			Ok(tx.send(None)?)
		} else {
			bail!("uninitialized ueberzugpp");
		}
	}

	// Currently Überzug++'s Wayland output only supports Sway, Hyprland and Wayfire
	// as it requires information from specific compositor socket directly.
	// These environment variables are from ueberzugpp src/canvas/wayland/config.cpp
	pub(crate) fn supported_compositor() -> bool {
		env_exists("SWAYSOCK")
			|| env_exists("HYPRLAND_INSTANCE_SIGNATURE")
			|| env_exists("WAYFIRE_SOCKET")
	}

	fn create_demon(adapter: Adapter) -> Result<Child> {
		let result = Command::new("ueberzugpp")
			.args(["layer", "-so", &adapter.to_string()])
			.env("SPDLOG_LEVEL", if cfg!(debug_assertions) { "debug" } else { "" })
			.kill_on_drop(true)
			.stdin(Stdio::piped())
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn();

		if let Err(ref e) = result {
			warn!("Failed to start ueberzugpp: {e}");
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

	async fn send_command(adapter: Adapter, child: &mut Child, cmd: Cmd) -> Result<()> {
		let s = if let Some((path, rect)) = cmd {
			debug!("ueberzugpp rect before adjustment: {:?}", rect);
			let rect = Self::adjust_rect(rect);
			debug!("ueberzugpp rect after adjustment: {:?}", rect);

			format!(
				r#"{{"action":"add","identifier":"yazi","x":{},"y":{},"max_width":{},"max_height":{},"path":"{}"}}{}"#,
				rect.x,
				rect.y,
				rect.width,
				rect.height,
				path.to_string_lossy(),
				'\n'
			)
		} else {
			format!(r#"{{"action":"remove","identifier":"yazi"}}{}"#, '\n')
		};

		debug!("`ueberzugpp layer -so {adapter}` command: {s}");
		child.stdin.as_mut().unwrap().write_all(s.as_bytes()).await?;

		Ok(())
	}
}
