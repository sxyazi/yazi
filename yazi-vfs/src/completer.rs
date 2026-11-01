use std::io;

use anyhow::{Context, Result, bail};
use yazi_config::VFS;
use yazi_fs::{engine::{DirReader, FileHolder}, path::clean_url};
use yazi_shared::{path::{DynPath, PathDyn, PathLike}, spec::{ParsedSpec, Spec}, strand::{AsStrand, Strand, StrandBuf, StrandLike}, url::{AsUrl, UrlBuf, UrlCow, UrlLike}};
use yazi_shim::bytes::{AnyAsciiChar, BytesExt};
use yazi_widgets::cmp::{CmpItem, CmpKey};

use crate::engine;

pub struct Completer<'a> {
	input:        &'a [u8],
	prefix:       Option<&'a [u8]>,
	child:        &'a [u8],
	trailing_sep: bool,
}

impl<'a> Completer<'a> {
	pub fn new<T>(input: &'a T) -> Self
	where
		T: AsRef<[u8]> + ?Sized,
	{
		let input = input.as_ref();
		let split = input.rsplit_pred_once(AnyAsciiChar::SEP);

		Self {
			input,
			prefix: split.map(|(parent, _)| &input[..=parent.len()]),
			child: split.map_or(input, |(_, child)| child),
			trailing_sep: split.is_some_and(|(_, child)| child.is_empty()),
		}
	}

	pub fn parse(&self) -> Result<(CmpKey, StrandBuf)> {
		let parsed = ParsedSpec::parse(self.input)?;
		if parsed.has_scheme() && !parsed.rest().contains(&b'/') {
			return Ok((CmpKey::Authorities, self.input.replace_seq_once(b"~://", b"://").into()));
		}

		let (parent, word) =
			if self.input.is_empty() { Self::new("./").split()? } else { self.split()? };
		let key = if self.prefix.is_some() {
			CmpKey::Directory(parent)
		} else {
			CmpKey::DirectoryWithAuthorities(parent)
		};

		Ok((key, word))
	}

	pub async fn complete(key: &CmpKey, prefix: impl AsStrand) -> io::Result<Vec<CmpItem>> {
		let prefix = prefix.as_strand();
		let (parent, mut cands) = match key {
			CmpKey::Authorities => return Ok(Self::authorities(prefix)),
			CmpKey::Directory(parent) => (parent, vec![]),
			CmpKey::DirectoryWithAuthorities(parent) => (parent, Self::authorities(prefix)),
		};

		// The filesystem root is itself a directory candidate.
		if parent.loc() == "/" && prefix.len() == 0 {
			cands.push(CmpItem {
				name:   Default::default(),
				value:  Some(Self::encode(parent, false)),
				is_dir: true,
			});
		}

		let mut it = engine::read_dir(parent).await?;
		while let Some(dent) = it.next().await? {
			let name = dent.name();
			if !name.starts_with(prefix) {
				continue;
			}

			let Ok(ty) = dent.file_type().await else { continue };
			let url = dent.url();

			cands.push(CmpItem {
				name:   name.into_owned(),
				value:  if url.is_rejoinable() { None } else { Some(Self::encode(&url, ty.is_dir())) },
				is_dir: ty.is_dir(),
			});
		}

		Ok(cands)
	}

	/// Returns a replacement, or None to keep the candidate's existing value.
	pub fn resolve(&self, key: &CmpKey, item: &CmpItem) -> Option<String> {
		if item.value.is_some() {
			return None;
		}

		let prefix = self.prefix.unwrap_or_default();
		let (prefix, sep) =
			if matches!(self.child, b"." | b"..") { (self.input, "/") } else { (prefix, "") };

		let parent = key.parent()?;
		if parent.is_regular()
			&& self.input.split_seq_once(b"://").is_none()
			&& let Some(prefix) = Self::plain(prefix)
			&& let Some(name) = Self::plain(item.name.encoded_bytes())
		{
			return Some(format!("{prefix}{sep}{name}{}", if item.is_dir { "/" } else { "" }));
		}

		Some(Self::encode(parent.try_join(&item.name).ok()?, item.is_dir))
	}

	fn split(&self) -> Result<(UrlBuf, StrandBuf)> {
		let (spec, path) = Spec::parse(self.input)?;
		if path.is_empty() && !self.trailing_sep {
			bail!("Missing completion path"); // We don't complete a `sftp://test`, but `sftp://test/`
		} else if spec.is_local() && path.as_strand() == "~" {
			bail!("Incomplete home directory"); // We don't complete a `~`, but `~/`
		}

		let child = path.rsplit_pred(AnyAsciiChar::SEP).map_or(path.dyn_path(), |(_, c)| c);
		let (parent, word) = if matches!(child.encoded_bytes(), b"." | b"..") {
			(path.dyn_path(), StrandBuf::default())
		} else {
			(path.try_strip_suffix(child)?, child.into())
		};

		let parent = self.resolve_parent(spec, path.dyn_path(), parent)?;
		let parent = if let Some(abs) = engine::try_absolute(&parent) { abs } else { parent };
		let parent = if parent.loc().is_empty() { parent.into_owned() } else { clean_url(parent) };
		Ok((parent, word))
	}

	fn resolve_parent<'p>(
		&'p self,
		spec: Spec,
		path: PathDyn<'p>,
		parent: PathDyn<'p>,
	) -> Result<UrlCow<'p>> {
		// Ordinary entries: the existing authority and ports still fit the parent path.
		if let Ok(url) = UrlCow::try_from((spec.clone(), parent)) {
			return Ok(url);
		}

		// Empty SFTP/Scope parents and Hub roots such as hub://old/@/fo keep the original prefix.
		if let Some(prefix) = self.prefix
			&& let Ok(prefix) = UrlCow::try_from(prefix)
			&& prefix.loc() == parent
		{
			return Ok(prefix);
		}

		// Hub domain changes and Mount/View port changes require URL-aware parent traversal.
		let url = UrlCow::try_from((spec, path))?;
		let spec = url.parent().context("No parent URL")?.spec();
		UrlCow::try_from((spec, parent))
	}

	fn authorities(prefix: Strand) -> Vec<CmpItem> {
		VFS
			.matcher()
			.authorities()
			.map(|auth| {
				if !auth.domain.is_catchall() {
					(auth.encode(false).to_string(), true)
				} else if auth.kind.is_hub() {
					(format!("{}:///@", auth.scheme), true)
				} else {
					(format!("{}://", auth.scheme), false)
				}
			})
			.filter(|(name, _)| name.as_strand().starts_with(prefix))
			.map(|(name, is_dir)| {
				let value = format!("{name}{}", if is_dir { "/" } else { "" });
				CmpItem { name: name.into(), value: Some(value), is_dir }
			})
			.collect()
	}

	fn encode(url: impl AsUrl, is_dir: bool) -> String {
		let url = url.as_url();
		let dir = if is_dir { "/" } else { "" };

		if Self::plain(url.loc().encoded_bytes()).is_some() {
			format!("{url}{dir}")
		} else {
			format!("{}{dir}", url.encode())
		}
	}

	fn plain(bytes: &[u8]) -> Option<&str> {
		str::from_utf8(bytes).ok().filter(|s| !s.bytes().any(|b| b.is_ascii_control()))
	}
}

#[cfg(test)]
mod tests {
	use yazi_shared::{strand::StrandLike, url::UrlLike};

	use super::*;

	fn compare(s: &str, parent: &str, child: &str) {
		let (mut p, c) = Completer::new(s).split().unwrap();
		if p.is_regular()
			&& let Ok(u) = p.try_strip_prefix(yazi_fs::CWD.load().as_ref())
		{
			p = u.as_os().unwrap().into();
		}
		assert_eq!((p, c.to_str().unwrap()), (parent.parse().unwrap(), child));
	}

	#[cfg(unix)]
	#[tokio::test]
	async fn test_split() {
		yazi_shim::init_tests();
		yazi_shared::init_tests();
		yazi_config::init_tests();
		yazi_fs::init();

		assert!(Completer::new("").split().is_err());
		assert!(Completer::new("sftp://vps").split().is_err());
		compare(" ", "", " ");

		compare("/", "/", "");
		compare("//", "/", "");
		compare("///", "/", "");

		compare("/foo", "/", "foo");
		compare("//foo", "/", "foo");
		compare("///foo", "/", "foo");

		compare("/foo/", "/foo/", "");
		compare("//foo/", "/foo/", "");
		compare("/foo/bar", "/foo/", "bar");
		compare("///foo/bar", "/foo/", "bar");

		compare("sftp://vps/a", "sftp://vps/", "a");
		compare("sftp://vps//a", "sftp://vps//", "a");
		compare("sftp://vps/foo%2Fba", "sftp://vps/", "foo%2Fba");
		compare("sftp~://vps/foo%2Fba", "sftp://vps/foo", "ba");

		compare("test-scope://aws/a", "test-scope://aws/", "a");
		compare("test-scope://aws//a", "test-scope://aws//", "a");

		compare("test-mount://7z:2:1/foo/ba", "test-mount://7z:1:1/foo", "ba");

		compare("test-hub://old/@/fo", "test-hub://old/@/", "fo");
		compare("test-hub~://old/@/%66o", "test-hub://old/@/", "fo");
		compare("test-hub://b1/@a1,root/foo/ba", "test-hub://a1/@root/foo", "ba");
		compare("test-hub~://b1/@a1,root/foo%2Fba", "test-hub://a1/@root/foo", "ba");
	}

	#[cfg(windows)]
	#[test]
	fn test_split() {
		yazi_shim::init_tests();
		yazi_shared::init_tests();
		yazi_config::init_tests();
		yazi_fs::init();
		compare("foo", "", "foo");

		compare(r"foo\", r"foo\", "");
		compare(r"foo\bar", r"foo\", "bar");
		compare(r"foo\bar\", r"foo\bar\", "");

		compare(r"C:\", r"C:\", "");
		compare(r"C:\foo", r"C:\", "foo");
		compare(r"C:\foo\", r"C:\foo\", "");
		compare(r"C:\foo\bar", r"C:\foo\", "bar");
	}
}
