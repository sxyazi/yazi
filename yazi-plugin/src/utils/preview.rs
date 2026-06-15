use mlua::{ExternalError, Function, IntoLuaMulti, Lua, Table, Value};
// use ratatui::style::Color;
use yazi_binding::{
	Error,
	elements::{Area, HighlightPosition, Renderable, Text},
};
use yazi_core::{Highlighter, MgrProxy, tab::PreviewLock};
use yazi_fs::FsUrl;
use yazi_runner::previewer::PeekError;
use yazi_shared::url::{AsUrl, UrlLike};

use super::Utils;

impl Utils {
	// TODO:
	// return (Text?, PeekError?) instead of (String, usize?) to align with other
	// APIs, and allow users to use the Text for more flexible preview
	pub(super) fn preview_code(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, t: Table| async move {
			let area: Area = t.raw_get("area")?;
			let position: Option<HighlightPosition> = t.raw_get("position").ok();

			let mut lock = PreviewLock::try_from(t)?;
			let path = lock.url.as_url().unified_path();

			let inner = match Highlighter::oneshot(path, lock.skip, area.size(), position).await {
				Ok(text) => {
					// tracing::debug!("{:?}", text);
					text
				}
				Err(e @ PeekError::Exceeded(max)) => return (e, max).into_lua_multi(&lua),
				Err(e) => {
					return e.into_lua_multi(&lua);
				}
			};

			// tracing::debug!("Inner: {inner}");

			lock.data = vec![Renderable::Text(Text { area, inner, ..Default::default() })];

			MgrProxy::update_peeked(lock);
			().into_lua_multi(&lua)
		})
	}

	// Note: You need to implement or update Highlighter::oneshot_with_highlight to accept line/column and highlight accordingly.

	pub(super) fn preview_widget(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|_, (t, value): (Table, Value)| async move {
			let mut lock = PreviewLock::try_from(t)?;
			lock.data = match value {
				Value::Nil => vec![],
				Value::Table(tbl) => tbl.sequence_values::<Renderable>().collect::<mlua::Result<_>>()?,
				Value::UserData(ud) => match Renderable::try_from(&ud) {
					Ok(r) => vec![r],
					Err(e) => {
						if let Ok(err) = ud.take::<Error>() {
							vec![
								Renderable::Clear(yazi_binding::elements::Clear { area: lock.area.into() }),
								Renderable::from(err).with_area(lock.area),
							]
						} else {
							Err(e)?
						}
					}
				},
				_ => Err("preview widget must be a renderable element or a table of them".into_lua_err())?,
			};

			MgrProxy::update_peeked(lock);
			Ok(())
		})
	}
}

// fn apply_highlight(text: &mut Text, column: usize, length: usize) {
// 	use ratatui::text::Span;
// 	if length == 0 {
// 		return;
// 	}
// 	let Some(line) = text.inner.lines.first_mut() else { return };
// 	let mut new_spans = Vec::new();
// 	let mut char_pos = 0usize;
// 	for span in std::mem::take(&mut line.spans) {
// 		let n = span.content.chars().count();
// 		let range = char_pos..char_pos + n;
// 		if range.end <= column || range.start >= column + length || n == 0 {
// 			new_spans.push(span);
// 		} else {
// 			let chars: Vec<char> = span.content.chars().collect();
// 			let hl_start = column.saturating_sub(char_pos).min(n);
// 			let hl_end = (column + length).saturating_sub(char_pos).min(n);
// 			if hl_start > 0 {
// 				new_spans.push(Span { content: chars[..hl_start].iter().collect(), style: span.style });
// 			}
// 			{
// 				let mut hl_style = span.style;
// 				hl_style.bg = hl_style.bg.or(Some(Color::Yellow));
// 				new_spans.push(Span { content: chars[hl_start..hl_end].iter().collect(), style: hl_style });
// 			}
// 			if hl_end < n {
// 				new_spans.push(Span { content: chars[hl_end..].iter().collect(), style: span.style });
// 			}
// 		}
// 		char_pos = range.end;
// 	}
// 	line.spans = new_spans;
// }
