use std::{collections::VecDeque, io, pin::Pin, sync::Arc, task::{Context, Poll}};

use object_store::{ObjectStore, path::Path};
use tokio::{io::{AsyncRead, AsyncSeek, AsyncWrite, AsyncWriteExt, BufWriter, ReadBuf}, sync::mpsc};
use typed_path::Component;
use yazi_config::vfs::{ServiceS3, Vfs};
use yazi_fs::provider::{Capabilities, FileBuilder, Provider};
use yazi_shared::{path::{AsPath, PathBufDyn}, strand::AsStrand, url::{AsUrl, Url, UrlBuf, UrlCow}};

use super::{DynStore, read_dir::ReadDir};

const PAGE_SIZE: usize = 500;
const COPY_BUF_SIZE: usize = 512 * 1024;
const COPY_CHUNK: usize = 64 * 1024;

#[derive(Clone)]
pub struct S3<'a> {
	url:    Url<'a>,
	key:    Path,
	store:  DynStore,
}

pub struct File;

#[derive(Clone, Copy, Default)]
pub struct Gate;

impl<'a> Provider for S3<'a> {
	type File = File;
	type Gate = Gate;
	type Me<'b> = S3<'b>;
	type ReadDir = ReadDir;
	type UrlCow = UrlCow<'a>;

	async fn absolute(&self) -> io::Result<Self::UrlCow> {
		Ok(if let Some(u) = super::absolute::try_absolute(self.url) { u } else { self.url.to_owned().into() })
	}

	async fn canonicalize(&self) -> io::Result<UrlBuf> { Ok(self.url.to_owned()) }

	fn capabilities(&self) -> Capabilities { Capabilities { symlink: false } }

	async fn casefold(&self) -> io::Result<UrlBuf> { Ok(self.url.to_owned()) }

	async fn copy<P>(&self, _to: P, _attrs: yazi_fs::provider::Attrs) -> io::Result<u64>
	where
		P: AsPath,
	{
		Err(io::Error::new(io::ErrorKind::Unsupported, "S3 provider is read-only"))
	}

	fn copy_with_progress<P, A>(&self, _to: P, _attrs: A) -> io::Result<tokio::sync::mpsc::Receiver<io::Result<u64>>>
	where
		P: AsPath,
		A: Into<yazi_fs::provider::Attrs>,
	{
		Err(io::Error::new(io::ErrorKind::Unsupported, "S3 provider is read-only"))
	}

	async fn create(&self) -> io::Result<Self::File> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "S3 provider is read-only"))
	}

	async fn create_dir(&self) -> io::Result<()> { Err(io::Error::new(io::ErrorKind::Unsupported, "S3 provider is read-only")) }

	async fn create_new(&self) -> io::Result<Self::File> {
		Err(io::Error::new(io::ErrorKind::Unsupported, "S3 provider is read-only"))
	}

	async fn hard_link<P>(&self, _to: P) -> io::Result<()>
	where
		P: AsPath,
	{ Err(io::Error::new(io::ErrorKind::Unsupported, "S3 provider is read-only")) }

	async fn metadata(&self) -> io::Result<yazi_fs::cha::Cha> {
		if self.key.to_string().is_empty() {
			return Ok(super::metadata::dir(self.url.name().unwrap_or_default()));
		}

		match self.store.head(&self.key).await {
			Ok(meta) => Ok(super::metadata::object(self.url.name().unwrap_or_default(), &meta)),
			Err(object_store::Error::NotFound { .. }) if self.dir_exists().await? => {
				Ok(super::metadata::dir(self.url.name().unwrap_or_default()))
			}
			Err(error) => Err(to_io(error)),
		}
	}

	async fn new<'b>(url: Url<'b>) -> io::Result<Self::Me<'b>> {
		match url {
			Url::S3 { loc, domain } => {
				let (_name, config) = Vfs::service::<&ServiceS3>(domain).await?;
				let (bucket, key) = split_bucket_and_key(loc.as_inner())?;
				let store = build_store(config, &bucket)?;
				Ok(Self::Me { url, key, store })
			}
			_ => Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Not an S3 URL: {url:?}"))),
		}
	}

	async fn open(&self) -> io::Result<Self::File> {
		Err(io::Error::new(
			io::ErrorKind::Unsupported,
			"S3 provider does not expose file handles",
		))
	}

	async fn read_dir(self) -> io::Result<Self::ReadDir> {
		let prefix = self.list_prefix();
		let dir = Arc::new(self.url.to_owned());
		Ok(ReadDir {
			dir,
			store: self.store,
			prefix,
			token: None,
			finished: false,
			page_size: PAGE_SIZE,
			buffer: VecDeque::new(),
		})
	}

	async fn read_link(&self) -> io::Result<PathBufDyn> { Err(io::Error::new(io::ErrorKind::Unsupported, "S3 has no symlinks")) }
	async fn remove_dir(&self) -> io::Result<()> { Ok(()) }
	async fn remove_file(&self) -> io::Result<()> { self.store.delete(&self.key).await.map_err(to_io) }
	async fn rename<P>(&self, _to: P) -> io::Result<()>
	where
		P: AsPath,
	{ Err(io::Error::new(io::ErrorKind::Unsupported, "S3 provider is read-only")) }
	async fn symlink<S, F>(&self, _original: S, _is_dir: F) -> io::Result<()>
	where
		S: AsStrand,
		F: AsyncFnOnce() -> io::Result<bool>,
	{ Err(io::Error::new(io::ErrorKind::Unsupported, "S3 has no symlinks")) }
	async fn symlink_metadata(&self) -> io::Result<yazi_fs::cha::Cha> { self.metadata().await }
	async fn trash(&self) -> io::Result<()> { Err(io::Error::new(io::ErrorKind::Unsupported, "S3 provider is read-only")) }
	fn url(&self) -> Url<'_> { self.url }

	async fn write<C>(&self, _contents: C) -> io::Result<()>
	where
		C: AsRef<[u8]>,
	{ Err(io::Error::new(io::ErrorKind::Unsupported, "S3 provider is read-only")) }
}

impl<'a> S3<'a> {
	pub(super) async fn read_bytes(&self) -> io::Result<Vec<u8>> {
		let bytes = self.store.get(&self.key).await.map_err(to_io)?.bytes().await.map_err(to_io)?;
		Ok(bytes.to_vec())
	}

	async fn dir_exists(&self) -> io::Result<bool> {
		use object_store::list::{PaginatedListOptions, PaginatedListStore};

		let prefix = self.list_prefix();
		let result = self
			.store
			.list_paginated(
				Some(&prefix),
				PaginatedListOptions {
					delimiter: Some("/".into()),
					max_keys: Some(1),
					..Default::default()
				},
			)
			.await
			.map_err(to_io)?;

		Ok(!result.result.common_prefixes.is_empty() || !result.result.objects.is_empty())
	}

	fn list_prefix(&self) -> String {
		let key = self.key.to_string();
		if key.is_empty() || key.ends_with('/') {
			key
		} else {
			format!("{key}/")
		}
	}
}

impl FileBuilder for Gate {
	type File = File;

	fn append(&mut self, _append: bool) -> &mut Self { self }

	fn attrs(&mut self, _attrs: yazi_fs::provider::Attrs) -> &mut Self { self }

	fn create(&mut self, _create: bool) -> &mut Self { self }

	fn create_new(&mut self, _create_new: bool) -> &mut Self { self }

	async fn open<U>(&self, _url: U) -> io::Result<Self::File>
	where
		U: yazi_shared::url::AsUrl,
	{
		Err(io::Error::new(
			io::ErrorKind::Unsupported,
			"S3 provider does not expose file handles",
		))
	}

	fn read(&mut self, _read: bool) -> &mut Self { self }

	fn truncate(&mut self, _truncate: bool) -> &mut Self { self }

	fn write(&mut self, _write: bool) -> &mut Self { self }
}

impl AsyncRead for File {
	fn poll_read(self: Pin<&mut Self>, _cx: &mut Context<'_>, _buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
		Poll::Ready(Err(io::Error::new(
			io::ErrorKind::Unsupported,
			"S3 provider does not expose file handles",
		)))
	}
}

impl AsyncSeek for File {
	fn start_seek(self: Pin<&mut Self>, _position: io::SeekFrom) -> io::Result<()> {
		Err(io::Error::new(
			io::ErrorKind::Unsupported,
			"S3 provider does not expose file handles",
		))
	}

	fn poll_complete(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
		Poll::Ready(Err(io::Error::new(
			io::ErrorKind::Unsupported,
			"S3 provider does not expose file handles",
		)))
	}
}

impl AsyncWrite for File {
	fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, _buf: &[u8]) -> Poll<io::Result<usize>> {
		Poll::Ready(Err(io::Error::new(
			io::ErrorKind::Unsupported,
			"S3 provider does not expose file handles",
		)))
	}

	fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		Poll::Ready(Err(io::Error::new(
			io::ErrorKind::Unsupported,
			"S3 provider does not expose file handles",
		)))
	}

	fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
		Poll::Ready(Err(io::Error::new(
			io::ErrorKind::Unsupported,
			"S3 provider does not expose file handles",
		)))
	}
}

pub(crate) async fn copy_impl(from: Url<'_>, to: Url<'_>, attrs: yazi_fs::provider::Attrs) -> io::Result<u64> {
	let provider = S3::new(from).await?;
	let bytes = provider.read_bytes().await?;
	let dist = crate::provider::create(to).await?;

	let mut writer = BufWriter::with_capacity(COPY_BUF_SIZE, dist);
	writer.write_all(&bytes).await?;
	writer.flush().await?;
	writer.get_ref().set_attrs(attrs).await.ok();
	writer.shutdown().await.ok();
	Ok(bytes.len() as u64)
}

pub(crate) fn copy_with_progress_impl(
	from: UrlBuf,
	to: UrlBuf,
	attrs: yazi_fs::provider::Attrs,
) -> mpsc::Receiver<io::Result<u64>> {
	let (tx, rx) = mpsc::channel(10);

	tokio::spawn(async move {
		let result = async {
			let provider = S3::new(from.as_url()).await?;
			let bytes = provider.read_bytes().await?;
			let dist = crate::provider::create(&to).await?;

			let mut writer = BufWriter::with_capacity(COPY_BUF_SIZE, dist);
			for chunk in bytes.chunks(COPY_CHUNK) {
				writer.write_all(chunk).await?;
				tx.send(Ok(chunk.len() as u64)).await.ok();
			}
			writer.flush().await?;

			let mut file = writer.into_inner();
			file.set_attrs(attrs).await.ok();
			file.shutdown().await.ok();
			Ok(())
		}
		.await;

		match result {
			Ok(()) => {
				tx.send(Ok(0)).await.ok();
			}
			Err(error) => {
				tx.send(Err(error)).await.ok();
			}
		}
	});

	rx
}

fn split_bucket_and_key(path: &typed_path::UnixPath) -> io::Result<(String, Path)> {
	let mut components = path.components().filter(|component| !component.is_root());
	let Some(bucket) = components.next() else {
		return Err(io::Error::new(
			io::ErrorKind::Unsupported,
			"S3 service root is not supported; use s3://<service>/<bucket>/",
		));
	};

	let bucket = String::from_utf8_lossy(bucket.as_bytes()).into_owned();
	if bucket.is_empty() {
		return Err(io::Error::new(
			io::ErrorKind::InvalidInput,
			"S3 bucket name must not be empty",
		));
	}

	let key = components
		.map(|component| String::from_utf8_lossy(component.as_bytes()).into_owned())
		.filter(|segment| !segment.is_empty())
		.collect::<Vec<_>>()
		.join("/");

	Ok((bucket, Path::from(key)))
}

fn build_store(config: &ServiceS3, bucket: &str) -> io::Result<DynStore> {
	let mut builder = object_store::aws::AmazonS3Builder::new().with_bucket_name(bucket);
	if let Some(region) = &config.region {
		builder = builder.with_region(region);
	}
	if let Some(endpoint) = &config.endpoint {
		builder = builder.with_endpoint(endpoint);
	}
	if let Some(key) = &config.access_key_id {
		builder = builder.with_access_key_id(key);
	}
	if let Some(secret) = &config.secret_access_key {
		builder = builder.with_secret_access_key(secret);
	}
	if let Some(token) = &config.session_token {
		builder = builder.with_token(token);
	}
	builder = builder.with_allow_http(config.allow_http);
	builder = builder.with_virtual_hosted_style_request(!config.force_path_style);
	Ok(Arc::new(builder.build().map_err(io::Error::other)?))
}

pub(super) fn to_io(error: object_store::Error) -> io::Error {
	match error {
		object_store::Error::NotFound { .. } => io::Error::from(io::ErrorKind::NotFound),
		object_store::Error::PermissionDenied { .. }
		| object_store::Error::Unauthenticated { .. } => io::Error::from(io::ErrorKind::PermissionDenied),
		other => io::Error::other(other),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use object_store::aws::AmazonS3Builder;
	use yazi_shared::loc::Loc;

	#[test]
	fn split_bucket_and_key_uses_first_segment_as_bucket() {
		let (bucket, key) = split_bucket_and_key(typed_path::UnixPath::new("/srgdata/foo/bar")).unwrap();
		assert_eq!(bucket, "srgdata");
		assert_eq!(key.to_string(), "foo/bar");
	}

	#[test]
	fn split_bucket_and_key_returns_empty_key_for_bucket_root() {
		let (bucket, key) = split_bucket_and_key(typed_path::UnixPath::new("/srgdata")).unwrap();
		assert_eq!(bucket, "srgdata");
		assert_eq!(key.to_string(), "");
	}

	#[test]
	fn provider_uses_bucket_root_as_empty_key() {
		let url = Url::S3 { loc: Loc::zeroed(typed_path::UnixPath::new("/srgdata")), domain: "yabos" };
		let s3 = S3 {
			url,
			key: Path::from(""),
			store: Arc::new(AmazonS3Builder::new().with_bucket_name("srgdata").with_region("us-east-1").build().unwrap()),
		};
		assert_eq!(s3.key.to_string(), "");
	}

	#[test]
	fn list_prefix_adds_trailing_slash_for_directories() {
		let url = Url::S3 { loc: Loc::zeroed(typed_path::UnixPath::new("/srgdata/data")), domain: "yabos" };
		let s3 = S3 {
			url,
			key: Path::from("data"),
			store: Arc::new(AmazonS3Builder::new().with_bucket_name("srgdata").with_region("us-east-1").build().unwrap()),
		};
		assert_eq!(s3.list_prefix(), "data/");
	}
}
