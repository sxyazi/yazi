use std::io;

use yazi_shared::{strand::{StrandBuf, StrandLike}, url::{AsUrl, UrlBuf, UrlLike}};
use yazi_shim::OptionExt;

use crate::engine;

pub async fn maybe_exists(url: impl AsUrl) -> bool {
	match engine::lstat(url).await {
		Ok(_) => true,
		Err(e) => e.kind() != io::ErrorKind::NotFound,
	}
}

pub async fn unique_dir(mut url: UrlBuf) -> io::Result<UrlBuf> {
	let mut err = match engine::create_dir(&url).await {
		Ok(()) => return Ok(url),
		Err(e) if e.kind() == io::ErrorKind::AlreadyExists => e,
		Err(e) => return Err(e),
	};

	let Some(name) = url.name().owned() else {
		return Err(io::Error::new(io::ErrorKind::InvalidInput, "empty directory name"));
	};

	let mut buf = StrandBuf::with_capacity(url.loc().kind(), name.len() + 5);
	for i in 1..=1000 {
		buf.clear();
		buf.try_push(&name)?;
		buf.push_str(format!("_{i}"));

		url.try_set_name(&buf)?;
		match engine::create_dir(&url).await {
			Ok(()) => return Ok(url),
			Err(e) if e.kind() == io::ErrorKind::AlreadyExists => err = e,
			Err(e) => return Err(e),
		}
	}

	Err(err)
}

pub async fn unique_file(mut url: UrlBuf) -> io::Result<engine::RwFile> {
	let mut err = match engine::create_file_new(&url).await {
		Ok(file) => return Ok(file),
		Err(e) if e.kind() == io::ErrorKind::AlreadyExists => e,
		Err(e) => return Err(e),
	};

	let Some(stem) = url.stem().owned() else {
		return Err(io::Error::new(io::ErrorKind::InvalidInput, "empty file stem"));
	};

	let ext = url.ext().owned();
	let mut buf = StrandBuf::with_capacity(
		url.loc().kind(),
		stem.len() + ext.as_ref().map_or(0, |e| e.len()) + 6,
	);

	for i in 1..=1000 {
		buf.clear();
		buf.try_push(&stem)?;
		buf.push_str(format!("_{i}"));

		if let Some(e) = &ext {
			buf.push_str(".");
			buf.try_push(e)?;
		}

		url.try_set_name(&buf)?;
		match engine::create_file_new(&url).await {
			Ok(file) => return Ok(file),
			Err(e) if e.kind() == io::ErrorKind::AlreadyExists => err = e,
			Err(e) => return Err(e),
		}
	}

	Err(err)
}
