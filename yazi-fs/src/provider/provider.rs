use std::{io, path::{Path, PathBuf}};

use tokio::io::{BufReader, BufWriter};
use twox_hash::XxHash3_128;
use yazi_shared::{scheme::SchemeRef, url::{Url, UrlBuf, UrlCow}};
use yazi_vfs::local::Xdg;

use crate::{cha::Cha, provider::{Provider, Providers, ReadDir, RwFile, local::{self, Local}}};

pub async fn absolute<'a, U>(url: U) -> io::Result<UrlCow<'a>>
where
	U: Into<Url<'a>>,
{
	let url: Url = url.into();
	Providers::new(url).await?.absolute(url).await
}

pub fn cache<'a, U>(url: U) -> Option<PathBuf>
where
	U: Into<Url<'a>>,
{
	let url: Url = url.into();
	match url.scheme {
		SchemeRef::Regular | SchemeRef::Search(_) => None,
		SchemeRef::Archive(name) => Some(
			Xdg::cache_dir()
				.join(format!("archive-{}", yazi_shared::url::Encode::domain(name)))
				.join(format!("{:x}", XxHash3_128::oneshot(url.loc.bytes()))),
		),
		SchemeRef::Sftp(name) => Some(
			Xdg::cache_dir()
				.join(format!("sftp-{}", yazi_shared::url::Encode::domain(name)))
				.join(format!("{:x}", XxHash3_128::oneshot(url.loc.bytes()))),
		),
	}
}

pub async fn calculate<'a, U>(url: U) -> io::Result<u64>
where
	U: Into<Url<'a>>,
{
	let url: Url = url.into();
	if let Some(path) = url.as_path() {
		local::SizeCalculator::total(path).await
	} else {
		super::SizeCalculator::total(url).await
	}
}

pub async fn canonicalize<'a, U>(url: U) -> io::Result<UrlBuf>
where
	U: Into<Url<'a>>,
{
	let url: Url = url.into();
	let canon = Providers::new(url).await?.canonicalize(url.loc).await?;

	Ok(match url.scheme {
		SchemeRef::Regular | SchemeRef::Search(_) => canon.into(),
		SchemeRef::Archive(_) => {
			Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem: archive"))?
		}
		SchemeRef::Sftp(_) => UrlBuf { loc: canon.into(), scheme: url.scheme.into() },
	})
}

pub async fn casefold<'a, U>(url: U) -> io::Result<UrlBuf>
where
	U: Into<Url<'a>>,
{
	let url: Url = url.into();
	let fold = Providers::new(url).await?.casefold(url.loc).await?;

	Ok(match url.scheme {
		SchemeRef::Regular | SchemeRef::Search(_) => fold.into(),
		SchemeRef::Archive(_) => {
			Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem: archive"))?
		}
		SchemeRef::Sftp(_) => UrlBuf { loc: fold.into(), scheme: url.scheme.into() },
	})
}

pub async fn copy<'a, U, V>(from: U, to: V, cha: Cha) -> io::Result<u64>
where
	U: Into<Url<'a>>,
	V: Into<Url<'a>>,
{
	let (from, to): (Url, Url) = (from.into(), to.into());

	match (from.as_path(), to.as_path()) {
		(Some(from), Some(to)) => Local.copy(from, to, cha).await,
		(None, None) if from.scheme.covariant(to.scheme) => {
			Providers::new(from).await?.copy(from.loc, to.loc, cha).await
		}
		(Some(_), None) | (None, Some(_)) | (None, None) => {
			let src = Providers::new(from).await?.open(from.loc).await?;
			let dist = Providers::new(to).await?.create(to.loc).await?;

			let mut reader = BufReader::with_capacity(524288, src);
			let mut writer = BufWriter::with_capacity(524288, dist);

			let written = tokio::io::copy(&mut reader, &mut writer).await?;
			writer.into_inner().set_cha(cha).await.ok();

			Ok(written)
		}
	}
}

pub async fn create<'a, U>(url: U) -> io::Result<RwFile>
where
	U: Into<Url<'a>>,
{
	let url: Url = url.into();
	Providers::new(url).await?.create(url.loc).await
}

pub async fn create_dir<'a, U>(url: U) -> io::Result<()>
where
	U: Into<Url<'a>>,
{
	let url: Url = url.into();
	Providers::new(url).await?.create_dir(url.loc).await
}

pub async fn create_dir_all<'a, U>(url: U) -> io::Result<()>
where
	U: Into<Url<'a>>,
{
	let url: Url = url.into();
	Providers::new(url).await?.create_dir_all(url.loc).await
}

pub async fn hard_link<'a, U, V>(original: U, link: V) -> io::Result<()>
where
	U: Into<Url<'a>>,
	V: Into<Url<'a>>,
{
	let (original, link): (Url, Url) = (original.into(), link.into());
	if original.scheme.covariant(link.scheme) {
		Providers::new(original).await?.hard_link(original.loc, link.loc).await
	} else {
		Err(io::Error::from(io::ErrorKind::CrossesDevices))
	}
}

pub async fn identical<'a, U, V>(a: U, b: V) -> io::Result<bool>
where
	U: Into<Url<'a>>,
	V: Into<Url<'a>>,
{
	if let (Some(a), Some(b)) = (a.into().as_path(), b.into().as_path()) {
		local::identical(a, b).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

pub async fn metadata<'a, U>(url: U) -> io::Result<Cha>
where
	U: Into<Url<'a>>,
{
	let url: Url = url.into();
	Providers::new(url).await?.metadata(url.loc).await
}

pub async fn must_identical<'a, U, V>(a: U, b: V) -> bool
where
	U: Into<Url<'a>>,
	V: Into<Url<'a>>,
{
	identical(a, b).await.unwrap_or(false)
}

pub async fn read_dir<'a, U>(url: U) -> io::Result<ReadDir>
where
	U: Into<Url<'a>>,
{
	let url: Url = url.into();
	Providers::new(url).await?.read_dir(url.loc).await
}

pub async fn read_link<'a, U>(url: U) -> io::Result<PathBuf>
where
	U: Into<Url<'a>>,
{
	let url: Url = url.into();
	Providers::new(url).await?.read_link(url.loc).await
}

pub async fn remove_dir<'a, U>(url: U) -> io::Result<()>
where
	U: Into<Url<'a>>,
{
	let url: Url = url.into();
	Providers::new(url).await?.remove_dir(url.loc).await
}

pub async fn remove_dir_all<'a, U>(url: U) -> io::Result<()>
where
	U: Into<Url<'a>>,
{
	let url: Url = url.into();
	Providers::new(url).await?.remove_dir_all(url.loc).await
}

pub async fn remove_file<'a, U>(url: U) -> io::Result<()>
where
	U: Into<Url<'a>>,
{
	let url: Url = url.into();
	Providers::new(url).await?.remove_file(url.loc).await
}

pub async fn rename<'a, U, V>(from: U, to: V) -> io::Result<()>
where
	U: Into<Url<'a>>,
	V: Into<Url<'a>>,
{
	let (from, to): (Url, Url) = (from.into(), to.into());
	if from.scheme.covariant(to.scheme) {
		Providers::new(from).await?.rename(from.loc, to.loc).await
	} else {
		Err(io::Error::from(io::ErrorKind::CrossesDevices))
	}
}

pub async fn symlink<'a, U, F>(original: &Path, link: U, is_dir: F) -> io::Result<()>
where
	U: Into<Url<'a>>,
	F: AsyncFnOnce() -> io::Result<bool>,
{
	let link: Url = link.into();
	Providers::new(link).await?.symlink(original, link.loc, is_dir).await
}

pub async fn symlink_dir<'a, U>(original: &Path, link: U) -> io::Result<()>
where
	U: Into<Url<'a>>,
{
	let link: Url = link.into();
	Providers::new(link).await?.symlink_dir(original, link.loc).await
}

pub async fn symlink_file<'a, U>(original: &Path, link: U) -> io::Result<()>
where
	U: Into<Url<'a>>,
{
	let link: Url = link.into();
	Providers::new(link).await?.symlink_file(original, link.loc).await
}

pub async fn symlink_metadata<'a, U>(url: U) -> io::Result<Cha>
where
	U: Into<Url<'a>>,
{
	let url: Url = url.into();
	Providers::new(url).await?.symlink_metadata(url.loc).await
}

pub async fn trash<'a, U>(url: U) -> io::Result<()>
where
	U: Into<Url<'a>>,
{
	let url: Url = url.into();
	Providers::new(url).await?.trash(url.loc).await
}

pub async fn write<'a, U, C>(url: U, contents: C) -> io::Result<()>
where
	U: Into<Url<'a>>,
	C: AsRef<[u8]>,
{
	let url: Url = url.into();
	Providers::new(url).await?.write(url.loc, contents).await
}
