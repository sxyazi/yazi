use std::{fs::{self, File, OpenOptions}, io::{self, BufReader, BufWriter, Write}, path::Path};

use anyhow::{Context, Result, ensure};
use ratatui_core::{buffer::Buffer, layout::Rect, style::{Modifier, Style}};
use yazi_actor::act;
use yazi_boot::BOOT;
use yazi_config::{LAYOUT, THEME, popup::ConfirmCfg};
use yazi_core::{Ctx, mgr::OpenOpt, tab::{FileStamp, View as DocumentView, cell_width, grapheme_text, graphemes}};
use yazi_macro::{emit, impl_data_any, relay, render, succ};
use yazi_proxy::ConfirmProxy;
use yazi_scheduler::NotifyProxy;
use yazi_shared::{data::Data, event::ActionCow, scope::Scope, url::{UrlBuf, UrlLike}};
use yazi_term::event::{KeyCode, KeyEvent, KeyEventKind};
use yazi_widgets::input::InputOpt;

#[derive(Clone, Debug)]
struct SaveResult {
	result: Result<Option<FileStamp>, String>,
	close:  bool,
}

impl_data_any!(SaveResult);

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

		if cx.tab().view.as_ref().is_some_and(DocumentView::protect) {
			succ!();
		}

		let cwd = cx.cwd().clone();
		let scope = cx.tab().scope.child();
		cx.tab_mut().view_load = Some(scope.clone());
		tokio::spawn(async move {
			let view = tokio::task::spawn_blocking({
				let (url, scope) = (url.clone(), scope.clone());
				move || Self::load(url, &scope)
			})
			.await
			.ok()
			.flatten();
			if scope.is_cancelled() {
				return;
			}

			let mut action = relay!(mgr:view_opened).with_scope(scope).with("cwd", cwd).with("url", url);
			if let Some(view) = view {
				action = action.with_any("view", view);
			}
			emit!(Call(action));
		});
		succ!();
	}

	fn load(url: UrlBuf, scope: &Scope) -> Option<DocumentView> {
		let mut options = OpenOptions::new();
		options.read(true);
		#[cfg(unix)]
		{
			use std::os::unix::fs::OpenOptionsExt;
			options.custom_flags(rustix::fs::OFlags::NONBLOCK.bits() as i32);
		}

		if scope.is_cancelled() {
			return None;
		}
		let file = options.open(url.as_local()?).ok()?;
		let meta = file.metadata().ok()?;
		if !meta.is_file() {
			return None;
		}

		let mut document =
			DocumentView::read(url, BufReader::with_capacity(64 * 1024, &file), scope).ok()?;
		let before = FileStamp::new(&meta).ok();
		let after = FileStamp::new(&file.metadata().ok()?).ok();
		document.stamp = before.filter(|_| before == after);
		Some(document)
	}

	pub(super) fn opened(cx: &mut Ctx, mut action: ActionCow) -> Result<Data> {
		cx.tab_mut().view_load = None;
		if let Some(view) = action.take_any("view") {
			if cx.tab().view.as_ref().is_some_and(DocumentView::protect) {
				succ!();
			}
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

	pub(super) fn action(cx: &mut Ctx, mut action: ActionCow) -> Result<Data> {
		match action.name.as_ref() {
			"command" => return Self::command(cx),
			"save" => return Self::save(cx, false),
			"saved" => return Self::saved(cx, action),
			"discard" => return Self::discard(cx),
			"discard_confirmed" => {
				let url: UrlBuf = action.take("url")?;
				if cx.tab().view.as_ref().is_some_and(|v| v.url == url && v.dirty() && !v.saving) {
					return Self::close(cx);
				}
				succ!();
			}
			_ => {}
		}

		let Some(view) = &mut cx.tab_mut().view else { succ!() };
		if action.name == "focus" {
			view.end_change();
			view.focused = !view.focused;
			succ!(render!());
		}

		if action.name == "close" {
			if view.insert {
				view.end_change();
				view.insert = false;
				succ!(render!());
			}
			if view.protect() {
				succ!();
			}
			return Self::close(cx);
		}

		let height = LAYOUT.get().view.height.saturating_sub(1).max(1) as isize;
		match action.name.as_ref() {
			"insert" if !view.saving => view.insert = true,
			"undo" if !view.saving => {
				let changed = view.undo();
				succ!(Self::redraw(view, changed));
			}
			"delete" if !view.saving => {
				let changed = view.delete(false);
				succ!(Self::redraw(view, changed));
			}
			"left" => view.horizontal(true),
			"right" => view.horizontal(false),
			"up" => view.vertical(-1),
			"down" => view.vertical(1),
			"page_up" => view.vertical(-height),
			"page_down" => view.vertical(height),
			"home" => view.line_edge(false),
			"end" => view.line_edge(true),
			"top" => view.move_to_line(0),
			"bottom" => view.move_to_line(view.line_count() - 1),
			_ => succ!(),
		}
		succ!(Self::redraw(view, true));
	}

	pub(super) fn insert_key(view: &mut DocumentView, key: &KeyEvent) -> bool {
		if !view.insert || key.kind == KeyEventKind::Release {
			return false;
		}

		let mut bytes = [0; 4];
		let changed = if key.modifiers.is_literal() && key.code == KeyCode::Backspace {
			!view.saving && view.delete(true)
		} else if key.modifiers.is_literal() && key.code == KeyCode::Enter {
			!view.saving && view.type_text(view.newline())
		} else if key.modifiers.is_literal() && key.code == KeyCode::Tab {
			!view.saving && view.type_text("\t")
		} else if let Some(text) = key.text(&mut bytes) {
			!view.saving
				&& if text.contains(char::is_control) { view.paste(text) } else { view.type_text(text) }
		} else {
			return false;
		};
		Self::redraw(view, changed);
		true
	}

	pub(super) fn paste(view: &mut DocumentView, text: &str) {
		let changed = view.insert && !view.saving && view.paste(text);
		Self::redraw(view, changed);
	}

	fn redraw(view: &mut DocumentView, changed: bool) {
		if changed {
			let area = LAYOUT.get().view;
			view.reveal(area.width as usize, area.height.saturating_sub(1) as usize);
			render!();
		}
	}

	fn close(cx: &mut Ctx) -> Result<Data> {
		let tab = cx.tab_mut();
		if let Some(scope) = tab.view_load.take() {
			scope.cancel();
		}
		succ!(render!(tab.view.take().is_some()));
	}

	fn command(cx: &mut Ctx) -> Result<Data> {
		let tab = cx.tab_mut();
		let Some(view) = tab.view.as_ref().filter(|view| view.focused && !view.insert) else {
			succ!();
		};
		let url = view.url.clone();
		let tab_id = tab.id;
		if let Some(load) = tab.view_load.take() {
			load.cancel();
		}

		act!(input:show, cx, InputOpt::default())?;
		cx.input.main.command_target = Some((tab_id, url));
		let area = cx.input.main_field_area(&cx.mgr);
		cx.input.main.repos(area);
		succ!();
	}

	pub(super) fn command_submit(cx: &mut Ctx, action: ActionCow) -> Result<Data> {
		let Some((tab_id, url)) = cx.input.main.command_target.take() else { succ!() };
		let command = cx.input.main.value().to_owned();
		act!(input:close, cx, action)?;

		let Some(view) = cx.tab().view.as_ref().filter(|view| view.url == url && view.focused) else {
			succ!();
		};
		if cx.tab != cx.mgr.tabs.cursor || cx.tab().id != tab_id || view.insert {
			succ!();
		}

		match command.trim() {
			"" => succ!(),
			"w" => Self::save(cx, false),
			"wq" => Self::save(cx, true),
			"q" => {
				if view.protect() {
					succ!();
				}
				Self::close(cx)
			}
			"q!" => {
				if view.saving {
					succ!(view.protect());
				}
				Self::close(cx)
			}
			other => succ!(NotifyProxy::push_warn("Unknown editor command", format!(":{other}"))),
		}
	}

	fn discard(cx: &mut Ctx) -> Result<Data> {
		let Some(view) = cx.tab().view.as_ref().filter(|v| !v.insert && v.dirty() && !v.saving) else {
			succ!();
		};
		let url = view.url.clone();
		let cfg = ConfirmCfg::discard_text(url.display().to_string());
		let scope = cx.tab().scope.clone();
		tokio::spawn(async move {
			if ConfirmProxy::show(cfg).await {
				emit!(Call(relay!(view:discard_confirmed).with_scope(scope).with("url", url)));
			}
		});
		succ!();
	}

	fn save(cx: &mut Ctx, close: bool) -> Result<Data> {
		if close && cx.tab().view.as_ref().is_some_and(|view| !view.dirty() && !view.saving) {
			return Self::close(cx);
		}

		if cx.tabs().iter().any(|tab| tab.view.as_ref().is_some_and(|v| v.saving)) {
			NotifyProxy::push_warn("Saving text", "Wait for the current save to finish.");
			succ!();
		}

		let scope = cx.tab().scope.clone();
		let Some(view) = cx.tab_mut().view.as_mut().filter(|v| v.dirty()) else { succ!() };
		let Some(stamp) = view.stamp else {
			NotifyProxy::push_error(
				"Save failed",
				"File changed or could not be verified. Changes are still in the editor.",
			);
			succ!();
		};
		let url = view.url.clone();
		let (text, bom) = view.snapshot();
		tokio::spawn(async move {
			let result = tokio::task::spawn_blocking(move || {
				let path = url.as_local().context("The file is no longer local")?;
				Self::write_file(path, stamp, |file| {
					if bom {
						file.write_all("\u{feff}".as_bytes())?;
					}
					text.write_to(file)
				})
			})
			.await
			.unwrap_or_else(|e| Err(e.into()));
			let result = SaveResult { result: result.map_err(|e| format!("{e:#}")), close };
			emit!(Call(relay!(view:saved).with_scope(scope).with_any("result", result)));
		});
		succ!(render!());
	}

	fn saved(cx: &mut Ctx, mut action: ActionCow) -> Result<Data> {
		let SaveResult { result, close } = action.take_any("result").context("Missing save result")?;
		let Some(view) = cx.tab_mut().view.as_mut() else { succ!() };
		view.saving = false;
		match result {
			Ok(stamp) => {
				view.mark_saved(stamp);
				if close {
					return Self::close(cx);
				}
			}
			Err(e) => {
				NotifyProxy::push_error("Save failed", format!("{e}. Changes are still in the editor."))
			}
		}
		succ!(render!());
	}

	fn write_file(
		path: &Path,
		stamp: FileStamp,
		write: impl FnOnce(&mut dyn Write) -> io::Result<()>,
	) -> Result<Option<FileStamp>> {
		let meta = Self::check_file(path, stamp)?;
		let dir =
			path.parent().filter(|p| !p.as_os_str().is_empty()).context("Missing parent directory")?;
		let mut opts = fs::OpenOptions::new();
		opts.write(true);
		#[cfg(unix)]
		{
			use std::os::unix::fs::OpenOptionsExt;

			use rustix::fs::OFlags;
			opts.custom_flags((OFlags::NOFOLLOW | OFlags::NONBLOCK).bits() as i32);
		}
		let original = opts.open(path).context("Cannot open the original file for writing")?;
		ensure!(FileStamp::new(&original.metadata()?)? == stamp, "The file changed on disk");

		let mut pending =
			tempfile::NamedTempFile::new_in(dir).context("Cannot create a temporary file")?;
		{
			let mut writer = BufWriter::with_capacity(64 * 1024, pending.as_file_mut());
			write(&mut writer).context("Cannot write the temporary file")?;
			writer.flush().context("Cannot flush the temporary file")?;
		}
		#[cfg(unix)]
		{
			use std::os::unix::fs::MetadataExt;

			use rustix::{fs::fchown, process::{Gid, Uid}};
			let current = pending.as_file().metadata()?;
			let uid = (meta.uid() != current.uid()).then(|| Uid::from_raw(meta.uid()));
			let gid = (meta.gid() != current.gid()).then(|| Gid::from_raw(meta.gid()));
			if uid.is_some() || gid.is_some() {
				fchown(pending.as_file(), uid, gid).context("Cannot preserve file ownership")?;
			}
		}
		pending
			.as_file()
			.set_permissions(meta.permissions())
			.context("Cannot preserve file permissions")?;
		#[cfg(target_os = "linux")]
		Self::copy_attributes(&original, pending.as_file())
			.context("Cannot preserve file attributes")?;
		pending.as_file().sync_all().context("Cannot sync the temporary file")?;

		Self::check_file(path, stamp)?;
		drop(original);
		let saved =
			pending.persist(path).map_err(|e| e.error).context("Cannot replace the original file")?;
		Ok(saved.metadata().ok().and_then(|m| FileStamp::new(&m).ok()))
	}

	fn check_file(path: &Path, stamp: FileStamp) -> Result<fs::Metadata> {
		let meta = fs::symlink_metadata(path).context("Cannot verify the original file")?;
		ensure!(meta.is_file(), "Saving requires a regular file, without a symbolic link");
		#[cfg(unix)]
		{
			use std::os::unix::fs::MetadataExt;
			ensure!(meta.nlink() == 1, "Saving a file with hard links would break the links");
		}
		ensure!(FileStamp::new(&meta)? == stamp, "The file changed on disk");
		Ok(meta)
	}

	#[cfg(target_os = "linux")]
	fn copy_attributes(original: &File, pending: &File) -> Result<()> {
		use rustix::{fs::{XattrFlags, fgetxattr, fgetxattr_query, flistxattr, flistxattr_query, fsetxattr}, io::Errno};

		let len = match flistxattr_query(original) {
			Err(Errno::OPNOTSUPP) => return Ok(()),
			result => result?,
		};
		if len == 0 {
			return Ok(());
		}
		let mut names = vec![0; len];
		let n = flistxattr(original, names.as_mut_slice())?;
		for name in names[..n].split(|&b| b == 0).filter(|s| !s.is_empty()) {
			let mut value = vec![0; fgetxattr_query(original, name)?];
			let n = fgetxattr(original, name, value.as_mut_slice())?;
			fsetxattr(pending, name, &value[..n], XattrFlags::empty())?;
		}
		Ok(())
	}

	pub(super) fn render(view: &DocumentView, area: Rect, buf: &mut Buffer) {
		let area = LAYOUT.get().view.intersection(area);
		if area.is_empty() {
			return;
		}

		let style = Style::default();
		let (mode, mode_style) = if view.saving {
			(" SAVING ", THEME.status.progress_label.get().0)
		} else if view.insert {
			(" -- INSERT -- ", THEME.mode.insert_main.get().0)
		} else {
			(" NORMAL ", THEME.mode.normal_main.get().0)
		};
		let mode_style = if view.focused { mode_style } else { style.add_modifier(Modifier::DIM) };
		let status_y = area.bottom() - 1;
		let marker = if view.dirty() { " * | " } else { " | " };
		let (x, _) = buf.set_stringn(area.x, status_y, mode, area.width as usize, mode_style);
		let (x, _) = buf.set_stringn(x, status_y, marker, (area.right() - x) as usize, style);
		if let Some(name) = view.url.name() {
			buf.set_stringn(x, status_y, name.to_string_lossy(), (area.right() - x) as usize, style);
		}

		let width = area.width as usize;
		let (top, left) = view.viewport(width, area.height as usize - 1);
		for (y, row) in (area.y..status_y).zip(top..view.line_count()) {
			let mut col = 0;
			for g in graphemes(view.line(row)) {
				let start = col;
				col += cell_width(g, col);
				if col > left + width {
					break;
				}
				if start >= left && col > start && g.char(0) != '\t' {
					buf.set_stringn(area.x + (start - left) as u16, y, grapheme_text(g), col - start, style);
				}
			}
		}
	}
}
