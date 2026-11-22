use anyhow::Result;
use ratatui_core::{buffer::Buffer, layout::Rect, style::{Modifier, Style}};
use tokio::{fs::OpenOptions, io::{AsyncBufReadExt, AsyncReadExt, BufReader}, select};
use yazi_actor::act;
use yazi_boot::BOOT;
use yazi_config::LAYOUT;
use yazi_core::{Ctx, mgr::OpenOpt, tab::View as DocumentView};
use yazi_macro::{emit, relay, render, succ};
use yazi_shared::{data::Data, event::{Action, ActionCow}, url::{UrlBuf, UrlLike}};

pub(super) struct View;

impl View {
	pub(super) fn open(cx: &mut Ctx, action: ActionCow) -> Result<Data> {
		if let Some(scope) = cx.tab_mut().view_load.take() {
			scope.cancel();
		}
		if BOOT.chooser_file.is_some() || !cx.tab().mode.is_normal() {
			return act!(mgr:open, cx, action);
		}

		let url = {
			let mut files = cx.tab().selected_or_hovered_files();
			match (files.next(), files.next()) {
				(Some(file), None) if file.is_file() && file.as_local().is_some() => Some(file.url.clone()),
				_ => None,
			}
		};
		let Some(url) = url else {
			return act!(mgr:open, cx, action);
		};

		if let Some(view) = &mut cx.tab_mut().view
			&& view.url == url
		{
			view.focused = true;
			succ!(render!());
		}

		let cwd = cx.cwd().clone();
		let scope = cx.tab().scope.child();
		cx.tab_mut().view_load = Some(scope.clone());
		tokio::spawn(async move {
			let view = select! {
				biased;
				_ = scope.cancelled() => return,
				view = Self::load(&url) => view,
			};

			let mut action = relay!(mgr:view_opened).with_scope(scope).with("cwd", cwd).with("url", url);
			if let Some(view) = view {
				action = action.with_any("view", view);
			}
			emit!(Call(action));
		});
		succ!();
	}

	async fn load(url: &UrlBuf) -> Option<DocumentView> {
		let mut options = OpenOptions::new();
		options.read(true);
		#[cfg(unix)]
		options.custom_flags(rustix::fs::OFlags::NONBLOCK.bits() as i32);

		let file = options.open(url.as_local()?).await.ok()?;
		if !file.metadata().await.ok()?.is_file() {
			return None;
		}

		let mut file = BufReader::new(file);
		let first = file.fill_buf().await.ok()?;
		if first.contains(&0) || std::str::from_utf8(first).is_err_and(|err| err.error_len().is_some())
		{
			return None;
		}

		let mut body = String::new();
		file.read_to_string(&mut body).await.ok()?;
		DocumentView::new(url.clone(), body)
	}

	pub(super) fn opened(cx: &mut Ctx, mut action: ActionCow) -> Result<Data> {
		cx.tab_mut().view_load = None;
		if let Some(view) = action.take_any("view") {
			if cx.tab == cx.mgr.tabs.cursor && cx.tab().view.is_none() {
				cx.preview_mut().reset();
			}
			cx.tab_mut().view = Some(view);
			succ!(render!());
		}

		act!(
			mgr:open,
			cx,
			OpenOpt {
				cwd:         Some(action.take("cwd")?),
				targets:     vec![action.take("url")?],
				interactive: false,
				hovered:     false,
			}
		)
	}

	pub(super) fn action(cx: &mut Ctx, action: &Action) -> Result<Data> {
		let tab = cx.tab_mut();
		if action.name == "close" {
			if let Some(scope) = tab.view_load.take() {
				scope.cancel();
			}
			succ!(render!(tab.view.take().is_some()));
		}

		let Some(view) = &mut tab.view else { succ!() };
		if action.name == "focus" {
			view.focused = !view.focused;
			succ!(render!());
		}

		let height = usize::from(LAYOUT.get().view.height.saturating_sub(1));
		let last = view.last_offset(height);
		let current = view.offset.min(last);
		let next = match action.name.as_ref() {
			"up" => current.saturating_sub(1),
			"down" => (current + 1).min(last),
			"page_up" => current.saturating_sub(height),
			"page_down" => current.saturating_add(height).min(last),
			"top" => 0,
			"bottom" => last,
			_ => succ!(),
		};
		view.offset = next;
		succ!(render!(next != current));
	}

	pub(super) fn render(view: &DocumentView, area: Rect, buf: &mut Buffer) {
		let area = LAYOUT.get().view.intersection(area);
		if area.is_empty() {
			return;
		}

		let style = Style::default();
		let title_style = if view.focused { style.add_modifier(Modifier::REVERSED) } else { style };
		let (x, _) = buf.set_stringn(area.x, area.y, "Read only | ", area.width as usize, title_style);
		if let Some(name) = view.url.name() {
			buf.set_stringn(x, area.y, name.to_string_lossy(), (area.right() - x) as usize, title_style);
		}

		for (y, line) in ((area.y + 1)..area.bottom()).zip(view.visible((area.height - 1) as usize)) {
			let mut x = area.x;
			for part in line.split_inclusive('\t') {
				let text = part.strip_suffix('\t').unwrap_or(part);
				(x, _) = buf.set_stringn(x, y, text, (area.right() - x) as usize, style);
				if part.ends_with('\t') {
					x = x.saturating_add(4 - (x - area.x) % 4).min(area.right());
				}
				if x == area.right() {
					break;
				}
			}
		}
	}
}
