#[cfg(unix)]
use std::os::unix::ffi::{OsStrExt, OsStringExt};
#[cfg(windows)]
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::{cell::Cell, ffi::{OsStr, OsString}, iter::{self, Peekable}, mem};

use yazi_shared::url::{AsUrl, Url, UrlCow};

use crate::FsUrl;

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

	fn selected(&self, tab: usize, idx: Option<usize>) -> impl Iterator<Item = Url<'_>>;

	fn hovered(&self, tab: usize) -> Option<Url<'_>>;

	fn yanked(&self) -> impl Iterator<Item = Url<'_>>;
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
	pub fn new(src: T) -> Self { Self { tab: src.tab() + 1, src } }

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
			Some('*') => self.visit_selected(it, buf), // TODO: remove this
			Some(c) if c.is_ascii_digit() => self.visit_digit(it, buf),
			_ => self.visit_unknown(it, buf),
		}
	}

	fn visit_selected(&mut self, it: &mut Iter, buf: &mut Buf) {
		let c = it.next().and_then(b2c);
		let idx = self.consume_digit(it);

		let mut first = true;
		for url in self.src.selected(self.tab, idx) {
			if !mem::replace(&mut first, false) {
				buf.push(b' ' as _);
			}

			if c == Some('S') {
				cue(buf, url.os_str());
			} else {
				cue(buf, url.unified_path_str());
			}
		}
		if first && idx.is_some() {
			cue(buf, "");
		}
	}

	fn visit_hovered(&mut self, it: &mut Iter, buf: &mut Buf) {
		match it.next().and_then(b2c) {
			Some('h') => {
				cue(buf, self.src.hovered(self.tab).map(|u| u.unified_path_str()).unwrap_or_default());
			}
			Some('H') => {
				cue(buf, self.src.hovered(self.tab).map(|u| u.os_str()).unwrap_or_default());
			}
			_ => unreachable!(),
		}
	}

	fn visit_dirname(&mut self, it: &mut Iter, buf: &mut Buf) {
		let c = it.next().and_then(b2c);
		let idx = self.consume_digit(it);

		let mut first = true;
		for url in self.src.selected(self.tab, idx) {
			if !mem::replace(&mut first, false) {
				buf.push(b' ' as _);
			}

			if c == Some('D') {
				cue(buf, url.parent().map(|p| p.os_str()).unwrap_or_default());
			} else {
				cue(buf, url.parent().map(|p| p.unified_path_str()).unwrap_or_default());
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

	fn visit_digit(&mut self, it: &mut Iter, buf: &mut Buf) {
		// TODO: remove
		match self.consume_digit(it) {
			Some(0) => {
				cue(buf, self.src.hovered(self.tab).map(|u| u.unified_path_str()).unwrap_or_default());
			}
			Some(n) => {
				cue(
					buf,
					self
						.src
						.selected(self.tab, Some(n))
						.next()
						.map(|u| u.unified_path_str())
						.unwrap_or_default(),
				);
			}
			None => unreachable!(),
		}
	}

	fn visit_yanked(&mut self, it: &mut Iter, buf: &mut Buf) {
		let c = it.next().and_then(b2c);

		let mut first = true;
		for url in self.src.yanked() {
			if !mem::replace(&mut first, false) {
				buf.push(b' ' as _);
			}

			if c == Some('Y') {
				cue(buf, url.os_str());
			} else {
				cue(buf, url.unified_path_str());
			}
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

			fn selected(&self, _tab: usize, idx: Option<usize>) -> impl Iterator<Item = Url<'_>> {
				if idx.is_none() {
					self.0.set(true);
				}
				iter::empty()
			}

			fn hovered(&self, _tab: usize) -> Option<Url<'_>> { None }

			fn yanked(&self) -> impl Iterator<Item = Url<'_>> {
				self.0.set(true);
				iter::empty()
			}
		}

		let src = Source(Cell::new(false));
		Splatter { src: &src, tab: 1 }.splat(cmd.as_ref());
		src.0.get()
	}
}

// TODO: remove
impl<'a, T> Splatable for &'a T
where
	T: AsRef<[UrlCow<'a>]>,
{
	fn tab(&self) -> usize { 0 }

	fn selected(&self, tab: usize, idx: Option<usize>) -> impl Iterator<Item = Url<'_>> {
		self
			.as_ref()
			.iter()
			.filter(move |_| tab == 1)
			.map(|u| u.as_url())
			.skip(idx.unwrap_or(1))
			.take(if idx.is_some() { 1 } else { usize::MAX })
	}

	fn hovered(&self, tab: usize) -> Option<Url<'_>> {
		self.as_ref().first().filter(|_| tab == 1).map(|u| u.as_url())
	}

	fn yanked(&self) -> impl Iterator<Item = Url<'_>> { iter::empty() }
}

#[cfg(test)]
mod tests {
	use super::*;

	struct Source(usize);

	impl Splatable for Source {
		fn tab(&self) -> usize { self.0 }

		fn selected(&self, tab: usize, mut idx: Option<usize>) -> impl Iterator<Item = Url<'_>> {
			let urls = if tab == 1 {
				vec![Url::regular("t1/s1"), Url::regular("t1/s2")]
			} else if tab == 2 {
				vec![Url::regular("t 2/s 1"), Url::regular("t 2/s 2")]
			} else {
				vec![]
			};

			idx = idx.and_then(|i| i.checked_sub(1));
			urls.into_iter().skip(idx.unwrap_or(0)).take(if idx.is_some() { 1 } else { usize::MAX })
		}

		fn hovered(&self, tab: usize) -> Option<Url<'_>> {
			if tab == 1 {
				Some(Url::regular("hovered"))
			} else if tab == 2 {
				Some(Url::regular("hover ed"))
			} else {
				None
			}
		}

		fn yanked(&self) -> impl Iterator<Item = Url<'_>> {
			[Url::regular("y1"), Url::regular("y 2"), Url::regular("y3")].into_iter()
		}
	}

	#[test]
	#[cfg(unix)]
	fn test_unix() {
		let cases = [
			// Selected
			(Source(0), r#"ls %s"#, r#"ls t1/s1 t1/s2"#),
			(Source(0), r#"ls %s1 %s2 %s3"#, r#"ls t1/s1 t1/s2 ''"#),
			(Source(0), r#"ls %s %s2 %s"#, r#"ls t1/s1 t1/s2 t1/s2 t1/s1 t1/s2"#),
			(Source(1), r#"ls %s"#, r#"ls 't 2/s 1' 't 2/s 2'"#),
			(Source(1), r#"ls %s1 %s3 %s2"#, r#"ls 't 2/s 1' '' 't 2/s 2'"#),
			(Source(2), r#"ls %s"#, r#"ls "#),
			(Source(2), r#"ls %s1 %s %s2"#, r#"ls ''  ''"#),
			// Hovered
			(Source(0), r#"ls %h"#, r#"ls hovered"#),
			(Source(1), r#"ls %h"#, r#"ls 'hover ed'"#),
			(Source(2), r#"ls %h"#, r#"ls ''"#),
			// Dirname
			(Source(0), r#"cd %d"#, r#"cd t1 t1"#),
			(Source(1), r#"cd %d"#, r#"cd 't 2' 't 2'"#),
			(Source(1), r#"cd %d1 %d3 %d2"#, r#"cd 't 2' '' 't 2'"#),
			(Source(2), r#"cd %d %d1"#, r#"cd  ''"#),
			// Yanked
			(Source(0), r#"cd %y"#, r#"cd y1 'y 2' y3"#),
			(Source(1), r#"cd %y"#, r#"cd y1 'y 2' y3"#),
			(Source(2), r#"cd %y"#, r#"cd y1 'y 2' y3"#),
			// Tab
			(Source(0), r#"ls %s %ts %s"#, r#"ls t1/s1 t1/s2 't 2/s 1' 't 2/s 2' t1/s1 t1/s2"#),
			(Source(1), r#"ls %s1 %ts %s2"#, r#"ls 't 2/s 1'  't 2/s 2'"#),
			(Source(1), r#"ls %s1 %Ts1 %s2 %Ts2"#, r#"ls 't 2/s 1' t1/s1 't 2/s 2' t1/s2"#),
			(Source(0), r#"ls %s1 %Ts1 %s2 %Ts2"#, r#"ls t1/s1 '' t1/s2 ''"#),
			(Source(0), r#"ls %ty"#, r#"ls y1 'y 2' y3"#),
			(Source(0), r#"ls %Ty"#, r#"ls y1 'y 2' y3"#),
			// Escape
			(
				Source(0),
				r#"echo % %% %s2 %%h %d %%%y %%%%ts %%%%%ts1"#,
				r#"echo % % t1/s2 %h t1 t1 %y1 'y 2' y3 %%ts %%'t 2/s 1'"#,
			),
			// TODO: remove
			(Source(0), r#"ls %1 %* %2 %0 %3"#, r#"ls t1/s1 t1/s1 t1/s2 t1/s2 hovered ''"#),
		];

		for (src, cmd, expected) in cases {
			let s = Splatter::new(src).splat(OsStr::new(cmd));
			assert_eq!(s, OsStr::new(expected), "{cmd}");
		}
	}
}
