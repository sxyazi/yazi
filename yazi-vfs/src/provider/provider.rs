use std::io;

use tokio::sync::mpsc;
use yazi_fs::{cha::Cha, provider::{Attrs, Capabilities, Provider, local::Local}};
use yazi_shared::{path::PathBufDyn, strand::AsStrand, url::{AsUrl, Url, UrlBuf, UrlCow}};

use super::{Providers, ReadDir, RwFile};

pub async fn absolute<'a, U>(url: &'a U) -> io::Result<UrlCow<'a>>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.absolute().await
}

pub async fn calculate<U>(url: U) -> io::Result<u64>
where
	U: AsUrl,
{
	let url = url.as_url();
	if let Some(path) = url.as_local() {
		yazi_fs::provider::local::SizeCalculator::total(path).await
	} else {
		super::SizeCalculator::total(url).await
	}
}

pub async fn canonicalize<U>(url: U) -> io::Result<UrlBuf>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.canonicalize().await
}

pub async fn capabilities<U>(url: U) -> io::Result<Capabilities>
where
	U: AsUrl,
{
	Ok(Providers::new(url.as_url()).await?.capabilities())
}

pub async fn casefold<U>(url: U) -> io::Result<UrlBuf>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.casefold().await
}

pub async fn copy<U, V>(from: U, to: V, attrs: Attrs) -> io::Result<u64>
where
	U: AsUrl,
	V: AsUrl,
{
	let (from, to) = (from.as_url(), to.as_url());

	match (from.kind().is_local(), to.kind().is_local()) {
		(true, true) => Local::new(from).await?.copy(to.loc(), attrs).await,
		(false, false) if from.scheme().covariant(to.scheme()) => {
			Providers::new(from).await?.copy(to.loc(), attrs).await
		}
		(true, false) | (false, true) | (false, false) => super::copy_impl(from, to, attrs).await,
	}
}

pub async fn copy_with_progress<U, V, A>(
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

	match (from.kind().is_local(), to.kind().is_local()) {
		(true, true) => Local::new(from).await?.copy_with_progress(to.loc(), attrs),
		(false, false) if from.scheme().covariant(to.scheme()) => {
			Providers::new(from).await?.copy_with_progress(to.loc(), attrs)
		}
		(true, false) | (false, true) | (false, false) => {
			Ok(super::copy_with_progress_impl(from.to_owned(), to.to_owned(), attrs.into()))
		}
	}
}

pub async fn create<U>(url: U) -> io::Result<RwFile>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.create().await
}

pub async fn create_dir<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.create_dir().await
}

pub async fn create_dir_all<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.create_dir_all().await
}

pub async fn create_new<U>(url: U) -> io::Result<RwFile>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.create_new().await
}

pub async fn hard_link<U, V>(original: U, link: V) -> io::Result<()>
where
	U: AsUrl,
	V: AsUrl,
{
	let (original, link) = (original.as_url(), link.as_url());
	if original.scheme().covariant(link.scheme()) {
		Providers::new(original).await?.hard_link(link.loc()).await
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
		yazi_fs::provider::local::identical(a, b).await
	} else {
		Err(io::Error::new(io::ErrorKind::Unsupported, "Unsupported filesystem"))
	}
}

pub async fn metadata<U>(url: U) -> io::Result<Cha>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.metadata().await
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
	Providers::new(url.as_url()).await?.open().await
}

pub async fn read_dir<U>(url: U) -> io::Result<ReadDir>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.read_dir().await
}

pub async fn read_link<U>(url: U) -> io::Result<PathBufDyn>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.read_link().await
}

pub async fn remove_dir<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.remove_dir().await
}

pub async fn remove_dir_all<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.remove_dir_all().await
}

pub async fn remove_dir_clean<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.remove_dir_clean().await
}

pub async fn remove_file<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.remove_file().await
}

pub async fn rename<U, V>(from: U, to: V) -> io::Result<()>
where
	U: AsUrl,
	V: AsUrl,
{
	let (from, to) = (from.as_url(), to.as_url());
	if from.scheme().covariant(to.scheme()) {
		Providers::new(from).await?.rename(to.loc()).await
	} else {
		Err(io::Error::from(io::ErrorKind::CrossesDevices))
	}
}

pub async fn symlink<U, S, F>(link: U, original: S, is_dir: F) -> io::Result<()>
where
	U: AsUrl,
	S: AsStrand,
	F: AsyncFnOnce() -> io::Result<bool>,
{
	Providers::new(link.as_url()).await?.symlink(original, is_dir).await
}

pub async fn symlink_dir<U, S>(link: U, original: S) -> io::Result<()>
where
	U: AsUrl,
	S: AsStrand,
{
	Providers::new(link.as_url()).await?.symlink_dir(original).await
}

pub async fn symlink_file<U, S>(link: U, original: S) -> io::Result<()>
where
	U: AsUrl,
	S: AsStrand,
{
	Providers::new(link.as_url()).await?.symlink_file(original).await
}

pub async fn symlink_metadata<U>(url: U) -> io::Result<Cha>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.symlink_metadata().await
}

pub async fn trash<U>(url: U) -> io::Result<()>
where
	U: AsUrl,
{
	Providers::new(url.as_url()).await?.trash().await
}

pub fn try_absolute<'a, U>(url: U) -> Option<UrlCow<'a>>
where
	U: Into<UrlCow<'a>>,
{
	let url = url.into();
	match url.as_url() {
		Url::Regular(_) | Url::Search { .. } => yazi_fs::provider::local::try_absolute(url),
		Url::Archive { .. } => None, // TODO
		Url::Sftp { .. } => crate::provider::sftp::try_absolute(url),
	}
}

pub async fn write<U, C>(url: U, contents: C) -> io::Result<()>
where
	U: AsUrl,
	C: AsRef<[u8]>,
{
	Providers::new(url.as_url()).await?.write(contents).await
}
