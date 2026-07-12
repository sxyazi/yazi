use std::io;

use tokio::sync::mpsc;
use yazi_fs::{cha::Cha, engine::{Attrs, Capabilities, Engine, local::Local}};
use yazi_shared::{path::PathBufDyn, strand::AsStrand, url::{AsUrl, Url, UrlBuf, UrlCow}};

use super::{Engines, ReadDir, RwFile};

pub async fn absolute<'a, U>(url: &'a U) -> io::Result<UrlCow<'a>>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.absolute().await
}

pub async fn calculate<U>(url: U) -> io::Result<u64>
where
	U: AsUrl,
{
	let url = url.as_url();
	if let Some(path) = url.as_local() {
		yazi_fs::engine::local::SizeCalculator::total(path).await
	} else {
		super::SizeCalculator::total(url).await
	}
}

pub async fn canonicalize<U>(url: U) -> io::Result<UrlBuf>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.canonicalize().await
}

pub async fn capabilities<U>(url: U) -> io::Result<Capabilities>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.capabilities().await
}

pub async fn casefold<U>(url: U) -> io::Result<UrlBuf>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.casefold().await
}

pub async fn copy<U, V>(from: U, to: V, attrs: Attrs) -> io::Result<u64>
where
	U: AsUrl,
	V: AsUrl,
{
	let (from, to) = (from.as_url(), to.as_url());

	match (from.kind().is_local(), to.kind().is_local()) {
		(true, true) => Local::new(from).await?.copy(to.loc(), attrs).await,
		(false, false) if from.auth().covariant(to.auth()) => {
			Engines::new(from).await?.copy(to.loc(), attrs).await
		}
		(true, false) | (false, true) | (false, false) => super::copy_impl(from, to, attrs).await,
	}
}

pub async fn copy_progressive<U, V, A>(
	from: U,
	to: V,
	attrs: A,
) -> io::Result<mpsc::Receiver<Result<u64, io::Error>>>
where
	U: AsUrl,
	V: AsUrl,
	A: Into<Attrs>,
{
	let (from, to) = (from.as_url(), to.as_url());
	let attrs = attrs.into();

	if from.auth().covariant(to.auth()) {
		let engine = Engines::new(from).await?;
		if engine.capabilities().await?.copy_progressive {
			return engine.copy_progressive(to.loc(), attrs);
		}
	}

	Ok(super::copy_progressive_impl(from.to_owned(), to.to_owned(), attrs))
}

pub async fn create<U>(url: U) -> io::Result<RwFile>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.create().await
}

pub async fn create_dir<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.create_dir().await
}

pub async fn create_dir_all<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.create_dir_all().await
}

pub async fn create_new<U>(url: U) -> io::Result<RwFile>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.create_new().await
}

pub async fn hard_link<U, V>(original: U, link: V) -> io::Result<()>
where
	U: AsUrl,
	V: AsUrl,
{
	let (original, link) = (original.as_url(), link.as_url());
	if original.auth().covariant(link.auth()) {
		Engines::new(original).await?.hard_link(link.loc()).await
	} else {
		Err(io::Error::from(io::ErrorKind::CrossesDevices))
	}
}

pub async fn identical<U, V>(a: U, b: V) -> io::Result<bool>
where
	U: AsUrl,
	V: AsUrl,
{
	if let (Some(a), Some(b)) = (a.as_url().as_local(), b.as_url().as_local()) {
		yazi_fs::engine::local::identical(a, b).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

pub async fn metadata<U>(url: U) -> io::Result<Cha>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.metadata().await
}

pub async fn must_identical<U, V>(a: U, b: V) -> bool
where
	U: AsUrl,
	V: AsUrl,
{
	identical(a, b).await.unwrap_or(false)
}

pub async fn open<U>(url: U) -> io::Result<RwFile>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.open().await
}

pub async fn read_dir<U>(url: U) -> io::Result<ReadDir>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.read_dir().await
}

pub async fn read_link<U>(url: U) -> io::Result<PathBufDyn>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.read_link().await
}

pub async fn remove_dir<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.remove_dir().await
}

pub async fn remove_dir_all<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.remove_dir_all().await
}

pub async fn remove_dir_clean<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.remove_dir_clean().await
}

pub async fn remove_file<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.remove_file().await
}

pub async fn rename<U, V>(from: U, to: V) -> io::Result<()>
where
	U: AsUrl,
	V: AsUrl,
{
	let (from, to) = (from.as_url(), to.as_url());
	if from.auth().covariant(to.auth()) {
		Engines::new(from).await?.rename(to.loc()).await
	} else {
		Err(io::Error::from(io::ErrorKind::CrossesDevices))
	}
}

pub async fn set_attrs<U>(url: U, attrs: Attrs) -> io::Result<()>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.set_attrs(attrs).await
}

pub async fn symlink<U, S, F>(link: U, original: S, is_dir: F) -> io::Result<()>
where
	U: AsUrl,
	S: AsStrand,
	F: AsyncFnOnce() -> io::Result<bool>,
{
	Engines::new(link.as_url()).await?.symlink(original, is_dir).await
}

pub async fn symlink_dir<U, S>(link: U, original: S) -> io::Result<()>
where
	U: AsUrl,
	S: AsStrand,
{
	Engines::new(link.as_url()).await?.symlink_dir(original).await
}

pub async fn symlink_file<U, S>(link: U, original: S) -> io::Result<()>
where
	U: AsUrl,
	S: AsStrand,
{
	Engines::new(link.as_url()).await?.symlink_file(original).await
}

pub async fn symlink_metadata<U>(url: U) -> io::Result<Cha>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.symlink_metadata().await
}

pub async fn trash<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.trash().await
}

pub fn try_absolute<'a, U>(url: U) -> Option<UrlCow<'a>>
where
	U: Into<UrlCow<'a>>,
{
	let url = url.into();
	match url.as_url() {
		Url::Regular(_) | Url::Search { .. } => yazi_fs::engine::local::try_absolute(url),
		Url::Mount { .. } | Url::Scope { .. } | Url::Sftp { .. } => super::try_absolute_impl(url),
	}
}

pub async fn write<U, C>(url: U, contents: C) -> io::Result<()>
where
	U: AsUrl,
	C: AsRef<[u8]>,
{
	Engines::new(url.as_url()).await?.write(contents).await
}
