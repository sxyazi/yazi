use std::{borrow::Cow, fs::Metadata, io::{self, Read}, ops::Range, time::SystemTime};

use ratatui_core::{buffer::CellWidth, layout::{Position, Rect}};
use ropey::{Rope, RopeSlice};
use unicode_segmentation::{GraphemeCursor, GraphemeIncomplete};
use yazi_macro::impl_data_any;
use yazi_scheduler::NotifyProxy;
use yazi_shared::{scope::Scope, url::{UrlBuf, UrlLike}};

#[derive(Clone, Debug)]
pub struct View {
	pub url:     UrlBuf,
	pub focused: bool,
	pub insert:  bool,
	pub saving:  bool,
	pub stamp:   Option<FileStamp>,

	text:    Rope,
	bom:     bool,
	crlf:    bool,
	cursor:  usize,
	column:  usize,
	goal:    Option<usize>,
	top:     usize,
	left:    usize,
	undo:    Vec<Change>,
	joining: bool,
	version: usize,
	saved:   usize,
	next:    usize,
}

#[derive(Clone, Debug)]
struct Change {
	cursor:  usize,
	version: usize,
	edits:   Vec<Edit>,
}

#[derive(Clone, Debug)]
struct Edit {
	start:    usize,
	inserted: usize,
	removed:  String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileStamp {
	len:      u64,
	modified: SystemTime,
	#[cfg(unix)]
	identity: (u64, u64, i64, i64),
}

impl FileStamp {
	pub fn new(meta: &Metadata) -> io::Result<Self> {
		#[cfg(unix)]
		use std::os::unix::fs::MetadataExt;

		Ok(Self {
			len:                   meta.len(),
			modified:              meta.modified()?,
			#[cfg(unix)]
			identity:              (meta.dev(), meta.ino(), meta.ctime(), meta.ctime_nsec()),
		})
	}
}

impl_data_any!(View);

impl View {
	pub fn read(url: UrlBuf, reader: impl Read, scope: &Scope) -> io::Result<Self> {
		let mut text = Rope::from_reader(TextReader { reader, scope })?;
		let bom = text.get_char(0) == Some('\u{feff}');
		if bom {
			text.remove(..1);
		}

		let first = text.line(0);
		let crlf = first.len_chars() >= 2
			&& first.char(first.len_chars() - 1) == '\n'
			&& first.char(first.len_chars() - 2) == '\r';
		Ok(Self {
			url,
			focused: true,
			insert: false,
			saving: false,
			stamp: None,
			text,
			bom,
			crlf,
			cursor: 0,
			column: 0,
			goal: None,
			top: 0,
			left: 0,
			undo: Vec::new(),
			joining: false,
			version: 0,
			saved: 0,
			next: 0,
		})
	}

	pub fn dirty(&self) -> bool { self.version != self.saved }

	pub fn protect(&self) -> bool {
		if self.saving {
			NotifyProxy::push_warn("Saving text", "Wait for the save to finish.");
		} else if self.dirty() {
			NotifyProxy::push_warn(
				"Unsaved changes",
				format!("{}: save with :w or discard with :q!.", self.url.display()),
			);
		} else {
			return false;
		}
		true
	}

	pub fn end_change(&mut self) { self.joining = false; }

	pub fn snapshot(&mut self) -> (Rope, bool) {
		self.end_change();
		self.saving = true;
		(self.text.clone(), self.bom)
	}

	pub fn mark_saved(&mut self, stamp: Option<FileStamp>) {
		self.stamp = stamp;
		self.saved = self.version;
	}

	pub fn newline(&self) -> &'static str { if self.crlf { "\r\n" } else { "\n" } }

	pub fn paste(&mut self, text: &str) -> bool {
		let mut clean = String::with_capacity(text.len());
		let mut chars = text.chars().peekable();
		while let Some(c) = chars.next() {
			match c {
				'\r' | '\n' => {
					if c == '\r' && chars.peek() == Some(&'\n') {
						chars.next();
					}
					clean.push_str(self.newline());
				}
				_ if c == '\t' || !c.is_control() => clean.push(c),
				_ => {}
			}
		}
		self.type_text(&clean)
	}

	pub fn type_text(&mut self, text: &str) -> bool {
		if text.is_empty() {
			return false;
		}
		self.replace(self.cursor..self.cursor, text);
		true
	}

	pub fn delete(&mut self, backward: bool) -> bool {
		let other = self.adjacent(backward);
		if other == self.cursor {
			return false;
		}
		self.replace(other.min(self.cursor)..other.max(self.cursor), "");
		true
	}

	fn replace(&mut self, range: Range<usize>, text: &str) {
		if !self.joining {
			self.undo.push(Change { cursor: self.cursor, version: self.version, edits: Vec::new() });
			self.next += 1;
			self.version = self.next;
			self.joining = self.insert;
		}

		let start = range.start;
		let removed =
			if range.is_empty() { String::new() } else { String::from(self.text.slice(range.clone())) };
		let inserted = text.chars().count();
		self.text.remove(range);
		self.text.insert(start, text);

		let edits = &mut self.undo.last_mut().unwrap().edits;
		if let Some(last) = edits.last_mut()
			&& last.start + last.inserted == start
			&& removed.is_empty()
		{
			last.inserted += inserted;
		} else {
			edits.push(Edit { start, inserted, removed });
		}

		let offset = self.text.char_to_byte(start + inserted);
		// Inserting or removing text can join the graphemes on either side of the edit.
		let slice = self.text.slice(..);
		let before = boundary(slice, offset, true);
		let after = boundary(slice, before, false);
		self.cursor = slice.byte_to_char(if offset == 0 { 0 } else { after });
		self.goal = None;
		self.update_column();
	}

	pub fn undo(&mut self) -> bool {
		self.end_change();
		let Some(change) = self.undo.pop() else { return false };
		for edit in change.edits.into_iter().rev() {
			self.text.remove(edit.start..edit.start + edit.inserted);
			self.text.insert(edit.start, &edit.removed);
		}
		self.cursor = change.cursor;
		self.version = change.version;
		self.goal = None;
		self.update_column();
		true
	}

	fn adjacent(&self, backward: bool) -> usize {
		let slice = self.text.slice(..);
		let offset = boundary(slice, slice.char_to_byte(self.cursor), backward);
		slice.byte_to_char(offset)
	}

	pub fn horizontal(&mut self, backward: bool) {
		self.end_change();
		self.cursor = self.adjacent(backward);
		self.goal = None;
		self.update_column();
	}

	pub fn line_edge(&mut self, end: bool) {
		self.end_change();
		let line = self.cursor_line();
		let start = self.text.line_to_char(line);
		self.cursor = if end { start + self.line(line).len_chars() } else { start };
		self.goal = None;
		self.update_column();
	}

	pub fn vertical(&mut self, delta: isize) {
		let line = self.cursor_line().saturating_add_signed(delta).min(self.line_count() - 1);
		self.move_to_line(line);
	}

	pub fn move_to_line(&mut self, line: usize) {
		self.end_change();
		let goal = *self.goal.get_or_insert(self.column);
		let mut offset = 0;
		let mut column = 0;
		for grapheme in graphemes(self.line(line)) {
			let next = column + cell_width(grapheme, column);
			if next > goal {
				break;
			}
			column = next;
			offset += grapheme.len_chars();
		}
		self.cursor = self.text.line_to_char(line) + offset;
		self.column = column;
	}

	fn cursor_line(&self) -> usize { self.text.char_to_line(self.cursor) }

	fn update_column(&mut self) {
		let start = self.text.line_to_char(self.cursor_line());
		let prefix = self.text.slice(start..self.cursor);
		self.column = graphemes(prefix).fold(0, |col, g| col + cell_width(g, col));
	}

	pub fn line_count(&self) -> usize { self.text.len_lines() }

	pub fn line(&self, index: usize) -> RopeSlice<'_> {
		let line = self.text.line(index);
		let mut end = line.len_chars();
		if end > 0 && line.char(end - 1) == '\n' {
			end -= 1;
			if end > 0 && line.char(end - 1) == '\r' {
				end -= 1;
			}
		}
		line.slice(..end)
	}

	pub fn reveal(&mut self, width: usize, height: usize) {
		(self.top, self.left) = self.viewport(width, height);
	}

	pub fn viewport(&self, width: usize, height: usize) -> (usize, usize) {
		let line = self.cursor_line();
		let top = self
			.top
			.clamp(line.saturating_sub(height.saturating_sub(1)), line)
			.min(self.line_count().saturating_sub(height.max(1)));
		let left = self.left.clamp(self.column.saturating_sub(width.saturating_sub(1)), self.column);
		(top, left)
	}

	pub fn screen_cursor(&self, area: Rect) -> Option<Position> {
		if area.width == 0 || area.height < 2 {
			return None;
		}
		let (top, left) = self.viewport(area.width as usize, area.height as usize - 1);
		Some(Position {
			x: area.x + (self.column - left) as u16,
			y: area.y + (self.cursor_line() - top) as u16,
		})
	}
}

struct TextReader<'a, R> {
	reader: R,
	scope:  &'a Scope,
}

impl<R: Read> Read for TextReader<'_, R> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		if self.scope.is_cancelled() {
			return Err(io::ErrorKind::Interrupted.into());
		}
		let len = self.reader.read(buf)?;
		if buf[..len].iter().any(|b| matches!(b, 0..=8 | 11..=12 | 14..=31 | 127)) {
			return Err(io::ErrorKind::InvalidData.into());
		}
		Ok(len)
	}
}

pub fn graphemes(slice: RopeSlice<'_>) -> impl Iterator<Item = RopeSlice<'_>> {
	let mut cursor = GraphemeCursor::new(0, slice.len_bytes(), true);
	std::iter::from_fn(move || {
		let start = cursor.cur_cursor();
		let end = seek_boundary(slice, &mut cursor, false)?;
		Some(slice.byte_slice(start..end))
	})
}

pub fn cell_width(grapheme: RopeSlice<'_>, column: usize) -> usize {
	let text = grapheme_text(grapheme);
	if text == "\t" { 4 - column % 4 } else { text.cell_width() as usize }
}

pub fn grapheme_text(grapheme: RopeSlice<'_>) -> Cow<'_, str> {
	if grapheme.char(0).is_control() && grapheme.char(0) != '\t' {
		Cow::Borrowed("�")
	} else {
		grapheme.into()
	}
}

fn boundary(slice: RopeSlice<'_>, offset: usize, backward: bool) -> usize {
	let mut cursor = GraphemeCursor::new(offset, slice.len_bytes(), true);
	seek_boundary(slice, &mut cursor, backward).unwrap_or(if backward {
		0
	} else {
		slice.len_bytes()
	})
}

fn seek_boundary(
	slice: RopeSlice<'_>,
	cursor: &mut GraphemeCursor,
	backward: bool,
) -> Option<usize> {
	let offset = cursor.cur_cursor();
	if (backward && offset == 0) || (!backward && offset == slice.len_bytes()) {
		return None;
	}
	let at = if backward { offset - 1 } else { offset };
	let (mut chunk, mut start, ..) = slice.chunk_at_byte(at);

	loop {
		let result = if backward {
			cursor.prev_boundary(chunk, start)
		} else {
			cursor.next_boundary(chunk, start)
		};
		match result {
			Ok(value) => return value,
			Err(GraphemeIncomplete::PrevChunk) => (chunk, start, _, _) = slice.chunk_at_byte(start - 1),
			Err(GraphemeIncomplete::NextChunk) => {
				(chunk, start, _, _) = slice.chunk_at_byte(start + chunk.len())
			}
			Err(GraphemeIncomplete::PreContext(end)) => {
				let (context, start, ..) = slice.chunk_at_byte(end - 1);
				cursor.provide_context(&context[..end - start], start);
			}
			Err(GraphemeIncomplete::InvalidOffset) => unreachable!(),
		}
	}
}
