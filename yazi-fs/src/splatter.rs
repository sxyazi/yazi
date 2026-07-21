#[cfg(unix)]
use std::os::unix::ffi::{OsStrExt, OsStringExt};
#[cfg(windows)]
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::{cell::Cell, ffi::{OsStr, OsString}, iter::{self, Peekable}, mem, path::Path};

use yazi_shared::url::UrlLike;

use crate::file::File;

#[cfg(unix)]
type Iter<'a> = Peekable<std::iter::Copied<std::slice::Iter<'a, u8>>>;
#[cfg(windows)]
type Iter<'a> = Peekable<std::os::windows::ffi::EncodeWide<'a>>;

#[cfg(unix)]
type Buf = Vec<u8>;
#[cfg(windows)]
type Buf = Vec<u16>;

#[derive(Clone, Copy)]
pub struct Splatter<T> {
	src: T,
	tab: usize,
}

pub trait Splatable {
	fn tab(&self) -> usize;

	fn selected(&self, tab: usize, idx: Option<usize>) -> impl Iterator<Item = &File>;

	fn hovered(&self, tab: usize) -> Option<&File>;

	fn yanked(&self, idx: Option<usize>) -> impl Iterator<Item = &File>;
}

#[cfg(unix)]
fn b2c(b: u8) -> Option<char> { Some(b as char) }
#[cfg(windows)]
fn b2c(b: u16) -> Option<char> { char::from_u32(b as u32) }

fn cue(buf: &mut Buf, s: impl AsRef<OsStr>) {
	#[cfg(unix)]
	buf.extend(yazi_shared::shell::escape_os_str(s.as_ref()).as_bytes());
	#[cfg(windows)]
	buf.extend(yazi_shared::shell::escape_os_str(s.as_ref()).encode_wide());
}

impl<T> Splatter<T>
where
	T: Splatable,
{
	pub fn new(src: T) -> Self { Self { tab: src.tab(), src } }

	pub fn splat(mut self, cmd: impl AsRef<OsStr>) -> OsString {
		#[cfg(unix)]
		let mut it = cmd.as_ref().as_bytes().iter().copied().peekable();
		#[cfg(windows)]
		let mut it = cmd.as_ref().encode_wide().peekable();

		let mut buf = vec![];
		while let Some(cur) = it.next() {
			if b2c(cur) == Some('%') && it.peek().is_some() {
				self.visit(&mut it, &mut buf);
			} else {
				buf.push(cur);
			}
		}

		#[cfg(unix)]
		return OsString::from_vec(buf);
		#[cfg(windows)]
		return OsString::from_wide(&buf);
	}

	fn visit(&mut self, it: &mut Iter, buf: &mut Buf) {
		let c = it.peek().copied().and_then(b2c);
		match c {
			Some('s') | Some('S') => self.visit_selected(it, buf),
			Some('h') | Some('H') => self.visit_hovered(it, buf),
			Some('d') | Some('D') => self.visit_dirname(it, buf),
			Some('t') | Some('T') => self.visit_tab(it, buf),
			Some('y') | Some('Y') => self.visit_yanked(it, buf),
			Some('%') => self.visit_escape(it, buf),
			_ => self.visit_unknown(it, buf),
		}
	}

	fn visit_selected(&mut self, it: &mut Iter, buf: &mut Buf) {
		let c = it.next().and_then(b2c);
		let idx = self.consume_digit(it);

		let mut first = true;
		for file in self.src.selected(self.tab, idx) {
			if !mem::replace(&mut first, false) {
				buf.push(b' ' as _);
			}

			if c == Some('S') {
				cue(buf, file.url.os_str());
			} else {
				cue(buf, &*file.content_path());
			}
		}
		if first && idx.is_some() {
			cue(buf, "");
		}
	}

	fn visit_hovered(&mut self, it: &mut Iter, buf: &mut Buf) {
		match it.next().and_then(b2c) {
			Some('h') => {
				cue(buf, &*self.src.hovered(self.tab).map(|f| f.content_path()).unwrap_or_default());
			}
			Some('H') => {
				cue(buf, self.src.hovered(self.tab).map(|f| f.url.os_str()).unwrap_or_default());
			}
			_ => unreachable!(),
		}
	}

	fn visit_dirname(&mut self, it: &mut Iter, buf: &mut Buf) {
		let c = it.next().and_then(b2c);
		let idx = self.consume_digit(it);

		let mut first = true;
		for file in self.src.selected(self.tab, idx) {
			if !mem::replace(&mut first, false) {
				buf.push(b' ' as _);
			}

			if c == Some('D') {
				cue(buf, file.url.parent().map(|p| p.os_str()).unwrap_or_default());
			} else {
				cue(buf, file.content_path().parent().unwrap_or(Path::new("")));
			}
		}
		if first && idx.is_some() {
			cue(buf, "");
		}
	}

	fn visit_tab(&mut self, it: &mut Iter, buf: &mut Buf) {
		let old = self.tab;
		match it.next().and_then(b2c) {
			Some('t') => self.tab = self.tab.saturating_add(1),
			Some('T') => self.tab = self.tab.saturating_sub(1),
			_ => unreachable!(),
		}

		self.visit(it, buf);
		self.tab = old;
	}

	fn visit_yanked(&mut self, it: &mut Iter, buf: &mut Buf) {
		let c = it.next().and_then(b2c);
		let idx = self.consume_digit(it);

		let mut first = true;
		for file in self.src.yanked(idx) {
			if !mem::replace(&mut first, false) {
				buf.push(b' ' as _);
			}

			if c == Some('Y') {
				cue(buf, file.url.os_str());
			} else {
				cue(buf, &*file.content_path());
			}
		}
		if first && idx.is_some() {
			cue(buf, "");
		}
	}

	fn visit_escape(&mut self, it: &mut Iter, buf: &mut Buf) { buf.push(it.next().unwrap()); }

	fn visit_unknown(&mut self, it: &mut Iter, buf: &mut Buf) {
		buf.push(b'%' as _);
		if let Some(b) = it.next() {
			buf.push(b);
		}
	}

	fn consume_digit(&mut self, it: &mut Iter) -> Option<usize> {
		fn next(it: &mut Iter) -> Option<usize> {
			let n = b2c(*it.peek()?)?.to_digit(10)? as usize;
			it.next();
			Some(n)
		}

		let mut sum = next(it)?;
		while let Some(n) = next(it) {
			sum = sum.checked_mul(10)?.checked_add(n)?;
		}
		Some(sum)
	}
}

impl<T> Splatter<T> {
	pub fn spread(cmd: impl AsRef<OsStr>) -> bool {
		struct Source(Cell<bool>);

		impl Splatable for &Source {
			fn tab(&self) -> usize { 0 }

			fn selected(&self, _tab: usize, idx: Option<usize>) -> impl Iterator<Item = &File> {
				if idx.is_none() {
					self.0.set(true);
				}
				iter::empty()
			}

			fn hovered(&self, _tab: usize) -> Option<&File> { None }

			fn yanked(&self, _idx: Option<usize>) -> impl Iterator<Item = &File> { iter::empty() }
		}

		let src = Source(Cell::new(false));
		Splatter { src: &src, tab: 1 }.splat(cmd.as_ref());
		src.0.get()
	}
}

impl<I> Splatable for &I
where
	I: ?Sized,
	for<'a> &'a I: IntoIterator<Item = &'a File>,
{
	fn tab(&self) -> usize { 1 }

	fn selected(&self, tab: usize, mut idx: Option<usize>) -> impl Iterator<Item = &File> {
		idx = idx.and_then(|i| i.checked_sub(1));
		(*self).into_iter().filter(move |_| tab == 1).skip(idx.unwrap_or(0)).take(if idx.is_some() {
			1
		} else {
			usize::MAX
		})
	}

	fn hovered(&self, _tab: usize) -> Option<&File> { None }

	fn yanked(&self, _idx: Option<usize>) -> impl Iterator<Item = &File> { iter::empty() }
}

#[cfg(test)]
mod tests {
	use std::sync::LazyLock;

	use super::*;

	struct Source(usize);

	static SELECTED: LazyLock<[Vec<File>; 2]> =
		LazyLock::new(|| [vec![file("t1/s1"), file("t1/s2")], vec![file("t 2/s 1"), file("t 2/s 2")]]);
	static HOVERED: LazyLock<[File; 2]> = LazyLock::new(|| [file("hovered"), file("hover ed")]);
	static YANKED: LazyLock<[File; 3]> = LazyLock::new(|| [file("y1"), file("y 2"), file("y3")]);

	fn file(path: &'static str) -> File { File::from_dummy(Path::new(path), None) }

	impl Splatable for Source {
		fn tab(&self) -> usize { self.0 }

		fn selected(&self, tab: usize, mut idx: Option<usize>) -> impl Iterator<Item = &File> {
			idx = idx.and_then(|i| i.checked_sub(1));
			tab
				.checked_sub(1)
				.and_then(|tab| SELECTED.get(tab))
				.into_iter()
				.flatten()
				.skip(idx.unwrap_or(0))
				.take(if idx.is_some() { 1 } else { usize::MAX })
		}

		fn hovered(&self, tab: usize) -> Option<&File> {
			tab.checked_sub(1).and_then(|tab| HOVERED.get(tab))
		}

		fn yanked(&self, mut idx: Option<usize>) -> impl Iterator<Item = &File> {
			idx = idx.and_then(|i| i.checked_sub(1));
			YANKED.iter().skip(idx.unwrap_or(0)).take(if idx.is_some() { 1 } else { usize::MAX })
		}
	}

	#[test]
	#[cfg(unix)]
	fn test_unix() {
		let cases = [
			// Selected
			(Source(1), r#"ls %s"#, r#"ls t1/s1 t1/s2"#),
			(Source(1), r#"ls %s1 %s2 %s3"#, r#"ls t1/s1 t1/s2 ''"#),
			(Source(1), r#"ls %s %s2 %s"#, r#"ls t1/s1 t1/s2 t1/s2 t1/s1 t1/s2"#),
			(Source(2), r#"ls %s"#, r#"ls 't 2/s 1' 't 2/s 2'"#),
			(Source(2), r#"ls %s1 %s3 %s2"#, r#"ls 't 2/s 1' '' 't 2/s 2'"#),
			(Source(3), r#"ls %s"#, r#"ls "#),
			(Source(3), r#"ls %s1 %s %s2"#, r#"ls ''  ''"#),
			// Hovered
			(Source(1), r#"ls %h"#, r#"ls hovered"#),
			(Source(2), r#"ls %h"#, r#"ls 'hover ed'"#),
			(Source(3), r#"ls %h"#, r#"ls ''"#),
			// Dirname
			(Source(1), r#"cd %d"#, r#"cd t1 t1"#),
			(Source(2), r#"cd %d"#, r#"cd 't 2' 't 2'"#),
			(Source(2), r#"cd %d1 %d3 %d2"#, r#"cd 't 2' '' 't 2'"#),
			(Source(3), r#"cd %d %d1"#, r#"cd  ''"#),
			// Yanked
			(Source(1), r#"cd %y"#, r#"cd y1 'y 2' y3"#),
			(Source(2), r#"cd %y"#, r#"cd y1 'y 2' y3"#),
			(Source(3), r#"cd %y"#, r#"cd y1 'y 2' y3"#),
			(Source(1), r#"cd %y1 %y3 %y2 %y4"#, r#"cd y1 y3 'y 2' ''"#),
			// Tab
			(Source(1), r#"ls %s %ts %s"#, r#"ls t1/s1 t1/s2 't 2/s 1' 't 2/s 2' t1/s1 t1/s2"#),
			(Source(2), r#"ls %s1 %ts %s2"#, r#"ls 't 2/s 1'  't 2/s 2'"#),
			(Source(2), r#"ls %s1 %Ts1 %s2 %Ts2"#, r#"ls 't 2/s 1' t1/s1 't 2/s 2' t1/s2"#),
			(Source(1), r#"ls %s1 %Ts1 %s2 %Ts2"#, r#"ls t1/s1 '' t1/s2 ''"#),
			(Source(1), r#"ls %ty"#, r#"ls y1 'y 2' y3"#),
			(Source(1), r#"ls %Ty"#, r#"ls y1 'y 2' y3"#),
			// Escape
			(
				Source(1),
				r#"echo % %% %s2 %%h %d %%%y %%%%ts %%%%%ts1"#,
				r#"echo % % t1/s2 %h t1 t1 %y1 'y 2' y3 %%ts %%'t 2/s 1'"#,
			),
		];

		for (src, cmd, expected) in cases {
			let s = Splatter::new(src).splat(OsStr::new(cmd));
			assert_eq!(s, OsStr::new(expected), "{cmd}");
		}
	}

	#[test]
	#[cfg(unix)]
	fn test_content_path() {
		use crate::file::FileExtra;

		let file = File {
			url:   Path::new("/logical/file").into(),
			cha:   Default::default(),
			extra: FileExtra::new(None, Some("/real/file".into())),
		};

		let s = Splatter::new(&[file]).splat(OsStr::new("%s %S %d %D"));
		assert_eq!(s, OsStr::new("/real/file /logical/file /real /logical"));
	}
}
