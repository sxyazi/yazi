use std::io;

use tokio::io::{AsyncWriteExt, BufReader, BufWriter};
use yazi_fs::{cha::Cha, provider::{Attrs, Provider, local::Local}};
use yazi_shared::{path::{AsPath, PathBufDyn}, url::{AsUrl, UrlBuf, UrlCow}};

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
		(true, false) | (false, true) | (false, false) => {
			let src = Providers::new(from).await?.open().await?;
			let dist = Providers::new(to).await?.create().await?;

			let mut reader = BufReader::with_capacity(524288, src);
			let mut writer = BufWriter::with_capacity(524288, dist);
			let written = tokio::io::copy(&mut reader, &mut writer).await?;

			writer.flush().await?;
			writer.get_ref().set_attrs(attrs).await.ok();
			writer.shutdown().await.ok();
			Ok(written)
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
	Ok(Providers::new(url.as_url()).await?.remove_dir_clean().await)
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

pub async fn symlink<U, V, F>(original: U, link: V, is_dir: F) -> io::Result<()>
where
	U: AsUrl,
	V: AsUrl,
	F: AsyncFnOnce() -> io::Result<bool>,
{
	let (original, link) = (original.as_url(), link.as_url());
	if original.scheme().covariant(link.scheme()) {
		Providers::new(link).await?.symlink(original.loc(), is_dir).await
	} else {
		Err(io::Error::from(io::ErrorKind::CrossesDevices))
	}
}

pub async fn symlink_dir<P, U>(original: P, link: U) -> io::Result<()>
where
	P: AsPath,
	U: AsUrl,
{
	Providers::new(link.as_url()).await?.symlink_dir(original).await
}

pub async fn symlink_file<P, U>(original: P, link: U) -> io::Result<()>
where
	P: AsPath,
	U: AsUrl,
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

pub async fn write<U, C>(url: U, contents: C) -> io::Result<()>
where
	U: AsUrl,
	C: AsRef<[u8]>,
{
	Providers::new(url.as_url()).await?.write(contents).await
}
