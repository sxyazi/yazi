use std::{io, path::{Path, PathBuf}};

use tokio::io::{BufReader, BufWriter};
use yazi_fs::{cha::Cha, provider::{Provider, local::Local}};
use yazi_shared::{scheme::SchemeRef, url::{AsUrl, UrlBuf, UrlCow}};

use super::{Providers, ReadDir, RwFile};

pub async fn absolute<'a, U>(url: &'a U) -> io::Result<UrlCow<'a>>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.absolute(url).await
}

pub async fn calculate<U>(url: U) -> io::Result<u64>
where
	U: AsUrl,
{
	let url = url.as_url();
	if let Some(path) = url.as_path() {
		yazi_fs::provider::local::SizeCalculator::total(path).await
	} else {
		super::SizeCalculator::total(url).await
	}
}

pub async fn canonicalize<U>(url: U) -> io::Result<UrlBuf>
where
	U: AsUrl,
{
	let url = url.as_url();
	let canon = Providers::new(url).await?.canonicalize(url.loc).await?;

	Ok(match url.scheme {
		SchemeRef::Regular | SchemeRef::Search(_) => canon.into(),
		SchemeRef::Archive(_) => {
			Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem: archive"))?
		}
		SchemeRef::Sftp(_) => UrlBuf { loc: canon.into(), scheme: url.scheme.into() },
	})
}

pub async fn casefold<U>(url: U) -> io::Result<UrlBuf>
where
	U: AsUrl,
{
	let url = url.as_url();
	let fold = Providers::new(url).await?.casefold(url.loc).await?;

	Ok(match url.scheme {
		SchemeRef::Regular | SchemeRef::Search(_) => fold.into(),
		SchemeRef::Archive(_) => {
			Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem: archive"))?
		}
		SchemeRef::Sftp(_) => UrlBuf { loc: fold.into(), scheme: url.scheme.into() },
	})
}

pub async fn copy<U, V>(from: U, to: V, cha: Cha) -> io::Result<u64>
where
	U: AsUrl,
	V: AsUrl,
{
	let (from, to) = (from.as_url(), to.as_url());

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

pub async fn create<U>(url: U) -> io::Result<RwFile>
where
	U: AsUrl,
{
	let url = url.as_url();
	Providers::new(url).await?.create(url.loc).await
}

pub async fn create_dir<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	let url = url.as_url();
	Providers::new(url).await?.create_dir(url.loc).await
}

pub async fn create_dir_all<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	let url = url.as_url();
	Providers::new(url).await?.create_dir_all(url.loc).await
}

pub async fn hard_link<U, V>(original: U, link: V) -> io::Result<()>
where
	U: AsUrl,
	V: AsUrl,
{
	let (original, link) = (original.as_url(), link.as_url());
	if original.scheme.covariant(link.scheme) {
		Providers::new(original).await?.hard_link(original.loc, link.loc).await
	} else {
		Err(io::Error::from(io::ErrorKind::CrossesDevices))
	}
}

pub async fn identical<U, V>(a: U, b: V) -> io::Result<bool>
where
	U: AsUrl,
	V: AsUrl,
{
	if let (Some(a), Some(b)) = (a.as_url().as_path(), b.as_url().as_path()) {
		yazi_fs::provider::local::identical(a, b).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

pub async fn metadata<U>(url: U) -> io::Result<Cha>
where
	U: AsUrl,
{
	let url = url.as_url();
	Providers::new(url).await?.metadata(url.loc).await
}

pub async fn must_identical<U, V>(a: U, b: V) -> bool
where
	U: AsUrl,
	V: AsUrl,
{
	identical(a, b).await.unwrap_or(false)
}

pub async fn read_dir<U>(url: U) -> io::Result<ReadDir>
where
	U: AsUrl,
{
	let url = url.as_url();
	Providers::new(url).await?.read_dir(url.loc).await
}

pub async fn read_link<U>(url: U) -> io::Result<PathBuf>
where
	U: AsUrl,
{
	let url = url.as_url();
	Providers::new(url).await?.read_link(url.loc).await
}

pub async fn remove_dir<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	let url = url.as_url();
	Providers::new(url).await?.remove_dir(url.loc).await
}

pub async fn remove_dir_all<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	let url = url.as_url();
	Providers::new(url).await?.remove_dir_all(url.loc).await
}

pub async fn remove_dir_clean<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	let url = url.as_url();
	Ok(Providers::new(url).await?.remove_dir_clean(url.loc).await)
}

pub async fn remove_file<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	let url = url.as_url();
	Providers::new(url).await?.remove_file(url.loc).await
}

pub async fn rename<U, V>(from: U, to: V) -> io::Result<()>
where
	U: AsUrl,
	V: AsUrl,
{
	let (from, to) = (from.as_url(), to.as_url());
	if from.scheme.covariant(to.scheme) {
		Providers::new(from).await?.rename(from.loc, to.loc).await
	} else {
		Err(io::Error::from(io::ErrorKind::CrossesDevices))
	}
}

pub async fn symlink<U, F>(original: &Path, link: U, is_dir: F) -> io::Result<()>
where
	U: AsUrl,
	F: AsyncFnOnce() -> io::Result<bool>,
{
	let link = link.as_url();
	Providers::new(link).await?.symlink(original, link.loc, is_dir).await
}

pub async fn symlink_dir<U>(original: &Path, link: U) -> io::Result<()>
where
	U: AsUrl,
{
	let link = link.as_url();
	Providers::new(link).await?.symlink_dir(original, link.loc).await
}

pub async fn symlink_file<U>(original: &Path, link: U) -> io::Result<()>
where
	U: AsUrl,
{
	let link = link.as_url();
	Providers::new(link).await?.symlink_file(original, link.loc).await
}

pub async fn symlink_metadata<U>(url: U) -> io::Result<Cha>
where
	U: AsUrl,
{
	let url = url.as_url();
	Providers::new(url).await?.symlink_metadata(url.loc).await
}

pub async fn trash<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	let url = url.as_url();
	Providers::new(url).await?.trash(url.loc).await
}

pub async fn write<U, C>(url: U, contents: C) -> io::Result<()>
where
	U: AsUrl,
	C: AsRef<[u8]>,
{
	let url = url.as_url();
	Providers::new(url).await?.write(url.loc, contents).await
}
