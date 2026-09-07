use std::io;

use yazi_fs::{cha::Cha, engine::{Attrs, Capabilities as C, Engine, Transmit}, file::File};
use yazi_shared::{path::PathBufDyn, strand::AsStrand, url::{AsUrl, UrlBuf, UrlCow, UrlLike}};

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

pub async fn capabilities<U>(url: U) -> io::Result<C>
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

pub async fn copy<U, V, A>(from: U, to: V, attrs: A) -> io::Result<Transmit>
where
	U: AsUrl,
	V: AsUrl,
	A: Into<Attrs>,
{
	let (from, to) = (from.as_url(), to.as_url());
	let attrs = attrs.into();

	let mut rx = Engines::new(from).await?.copy_to(to, attrs).await?;
	if rx.is_supported().await {
		return Ok(rx);
	}

	let mut rx = Engines::new(to).await?.copy_from(from, attrs).await?;
	if rx.is_supported().await {
		return Ok(rx);
	}

	Ok(super::copy_progressive_impl(from.into(), to.into(), attrs))
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

pub async fn file<U>(url: U) -> io::Result<File>
where
	U: AsUrl,
{
	Engines::new(url.as_url()).await?.file().await
}

pub async fn hard_link<U, V>(original: U, link: V) -> io::Result<()>
where
	U: AsUrl,
	V: AsUrl,
{
	let (original, link) = (original.as_url(), link.as_url());
	if original.auth().same_service(link.auth()) {
		Engines::new(original).await?.hard_link(link.loc()).await
	} else {
		Err(io::ErrorKind::CrossesDevices.into())
	}
}

async fn identical<U, V>(a: U, b: V) -> io::Result<bool>
where
	U: AsUrl,
	V: AsUrl,
{
	match (a.as_url().as_local(), b.as_url().as_local()) {
		(Some(a), Some(b)) => yazi_fs::engine::local::identical(a, b).await,
		_ => Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem")),
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
	identical(a, b).await.unwrap_or_default()
}

pub(crate) async fn open<U>(url: U) -> io::Result<RwFile>
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

pub(crate) async fn revalidate(file: &File) -> io::Result<Option<File>> {
	Engines::new(file.as_url()).await?.revalidate(file.clone()).await
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
	if from.auth().same_service(to.auth()) {
		Engines::new(from).await?.rename(to.loc()).await
	} else {
		Err(io::ErrorKind::CrossesDevices.into())
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
	if url.is_regular() {
		yazi_fs::engine::local::try_absolute(url)
	} else {
		super::try_absolute_impl(url)
	}
}

pub async fn write<U, C>(url: U, contents: C) -> io::Result<()>
where
	U: AsUrl,
	C: AsRef<[u8]>,
{
	Engines::new(url.as_url()).await?.write(contents).await
}
