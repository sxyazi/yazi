use std::{ffi::{OsStr, OsString}, iter::{self, Peekable}, mem, path::Path};

use yazi_shared::url::UrlLike;
use yazi_shim::wtf8::{AsWtf8, FromWtf8Vec};

use crate::{file::{File, FileRef}, files::FileIter};

type Iter<'a> = Peekable<std::iter::Copied<std::slice::Iter<'a, u8>>>;

#[derive(Clone, Copy)]
pub struct Splatter<T> {
	src: T,
	tab: usize,
}

pub trait Splatable {
	fn tab(&self) -> usize;

	fn selected(&self, tab: usize) -> mlua::Result<FileIter<impl Iterator<Item = &File>>>;

	fn hovered(&self, _tab: usize) -> mlua::Result<Option<FileRef<'_>>> { Ok(None) }

	fn yanked(&self) -> mlua::Result<FileIter<impl Iterator<Item = &File>>> {
		Ok(FileIter::new(iter::empty()))
	}
}

fn cue(buf: &mut Vec<u8>, s: impl AsRef<OsStr>) {
	yazi_shared::shell::escape_os_bytes_into(s.as_ref().as_wtf8(), buf);
}

impl<T: Splatable> Splatter<T> {
	pub fn new(src: T) -> Self { Self { tab: src.tab(), src } }

	pub fn splat(mut self, cmd: impl AsRef<OsStr>) -> mlua::Result<OsString> {
		let mut it = cmd.as_ref().as_wtf8().iter().copied().peekable();

		let mut buf = vec![];
		while let Some(cur) = it.next() {
			if cur == b'%' && it.peek().is_some() {
				self.visit(&mut it, &mut buf)?;
			} else {
				buf.push(cur);
			}
		}

		Ok(OsString::from_wtf8_vec(buf).expect("valid WTF-8"))
	}

	fn visit(&mut self, it: &mut Iter, buf: &mut Vec<u8>) -> mlua::Result<()> {
		match it.peek().copied() {
			Some(b's' | b'S') => self.visit_selected(it, buf),
			Some(b'h' | b'H') => self.visit_hovered(it, buf),
			Some(b'd' | b'D') => self.visit_dirname(it, buf),
			Some(b't' | b'T') => self.visit_tab(it, buf),
			Some(b'y' | b'Y') => self.visit_yanked(it, buf),
			Some(b'%') => self.visit_escape(it, buf),
			_ => self.visit_unknown(it, buf),
		}
	}

	fn visit_selected(&mut self, it: &mut Iter, buf: &mut Vec<u8>) -> mlua::Result<()> {
		let c = it.next().unwrap();
		let idx = Self::consume_digit(it);

		Self::visit_files(self.src.selected(self.tab)?, c, idx, buf)
	}

	fn visit_hovered(&mut self, it: &mut Iter, buf: &mut Vec<u8>) -> mlua::Result<()> {
		let c = it.next().unwrap();
		if let Some(file) = self.src.hovered(self.tab)? {
			file.borrow(|f| Ok(Self::visit_file(f, c, buf)))?;
		} else {
			cue(buf, "");
		}
		Ok(())
	}

	fn visit_dirname(&mut self, it: &mut Iter, buf: &mut Vec<u8>) -> mlua::Result<()> {
		self.visit_selected(it, buf)
	}

	fn visit_tab(&mut self, it: &mut Iter, buf: &mut Vec<u8>) -> mlua::Result<()> {
		let old = self.tab;
		match it.next() {
			Some(b't') => self.tab = self.tab.saturating_add(1),
			Some(b'T') => self.tab = self.tab.saturating_sub(1),
			_ => unreachable!(),
		}

		let result = self.visit(it, buf);
		self.tab = old;
		result
	}

	fn visit_yanked(&mut self, it: &mut Iter, buf: &mut Vec<u8>) -> mlua::Result<()> {
		let c = it.next().unwrap();
		let idx = Self::consume_digit(it);

		Self::visit_files(self.src.yanked()?, c, idx, buf)
	}

	fn visit_escape(&mut self, it: &mut Iter, buf: &mut Vec<u8>) -> mlua::Result<()> {
		buf.push(it.next().unwrap());
		Ok(())
	}

	fn visit_unknown(&mut self, it: &mut Iter, buf: &mut Vec<u8>) -> mlua::Result<()> {
		buf.push(b'%');
		if let Some(b) = it.next() {
			buf.push(b);
		}
		Ok(())
	}

	fn visit_file(file: &File, c: u8, buf: &mut Vec<u8>) {
		match c {
			b's' | b'h' | b'y' => cue(buf, &*file.content_path()),
			b'S' | b'H' | b'Y' => cue(buf, file.os_str()),
			b'd' => cue(buf, file.content_path().parent().unwrap_or(Path::new(""))),
			b'D' => cue(buf, file.parent().map(|p| p.os_str()).unwrap_or_default()),
			_ => unreachable!(),
		}
	}

	fn visit_files<'a>(
		files: FileIter<impl Iterator<Item = &'a File>>,
		c: u8,
		idx: Option<usize>,
		buf: &mut Vec<u8>,
	) -> mlua::Result<()> {
		let mut first = true;
		let visit = |file: &File| {
			if !mem::replace(&mut first, false) {
				buf.push(b' ');
			}
			Self::visit_file(file, c, buf);
			Ok(())
		};

		if let Some(i) = idx.and_then(|i| i.checked_sub(1)) {
			files.try_visit(i, visit)?;
		} else {
			files.try_for_each(visit)?;
		}

		if first && idx.is_some() {
			cue(buf, "");
		}

		Ok(())
	}
}

impl<T> Splatter<T> {
	pub fn spread(cmd: impl AsRef<OsStr>) -> mlua::Result<bool> {
		let mut it = cmd.as_ref().as_wtf8().iter().copied().peekable();
		while let Some(c) = it.next() {
			if c != b'%' {
				continue;
			}
			while matches!(it.peek(), Some(b't' | b'T')) {
				it.next();
			}
			if matches!(it.next(), Some(b's' | b'S' | b'd' | b'D'))
				&& Self::consume_digit(&mut it).unwrap_or(0) == 0
			{
				return Ok(true);
			}
		}
		Ok(false)
	}

	fn consume_digit(it: &mut Iter) -> Option<usize> {
		fn next(it: &mut Iter) -> Option<usize> {
			let n = char::from(*it.peek()?).to_digit(10)? as usize;
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

impl<I> Splatable for &I
where
	I: ?Sized,
	for<'a> &'a I: IntoIterator<Item = &'a File>,
{
	fn tab(&self) -> usize { 1 }

	fn selected(&self, tab: usize) -> mlua::Result<FileIter<impl Iterator<Item = &File>>> {
		Ok(FileIter::new((*self).into_iter().filter(move |_| tab == 1)))
	}
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

		fn selected(&self, tab: usize) -> mlua::Result<FileIter<impl Iterator<Item = &File>>> {
			Ok(FileIter::new(tab.checked_sub(1).and_then(|tab| SELECTED.get(tab)).into_iter().flatten()))
		}

		fn hovered(&self, tab: usize) -> mlua::Result<Option<FileRef<'_>>> {
			Ok(tab.checked_sub(1).and_then(|tab| HOVERED.get(tab)).map(Into::into))
		}

		fn yanked(&self) -> mlua::Result<FileIter<impl Iterator<Item = &File>>> {
			Ok(FileIter::new(YANKED.iter()))
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
			let s = Splatter::new(src).splat(OsStr::new(cmd)).unwrap();
			assert_eq!(s, OsStr::new(expected), "{cmd}");
		}
	}

	#[test]
	#[cfg(unix)]
	fn test_content_path() {
		use crate::file::FileExtra;

		let file = File {
			url:   Path::new("/logical/file").into(),
			stat:  Default::default(),
			extra: FileExtra::new(Default::default(), None, Some("/real/file".into())),
		};

		let s = Splatter::new(&[file]).splat(OsStr::new("%s %S %d %D")).unwrap();
		assert_eq!(s, OsStr::new("/real/file /logical/file /real /logical"));
	}
}
