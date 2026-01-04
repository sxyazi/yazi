use std::io::{self};

use yazi_macro::ok_or_not_found;
use yazi_shared::{strand::{StrandBuf, StrandLike}, url::{AsUrl, UrlBuf, UrlLike}};

use crate::provider;

#[inline]
pub async fn maybe_exists(url: impl AsUrl) -> bool {
	match provider::symlink_metadata(url).await {
		Ok(_) => true,
		Err(e) => e.kind() != io::ErrorKind::NotFound,
	}
}

// TODO: deprecate
pub async fn unique_name<F>(u: UrlBuf, append: F) -> io::Result<UrlBuf>
where
	F: Future<Output = bool>,
{
	match provider::symlink_metadata(&u).await {
		Ok(_) => _unique_name(u, append.await).await,
		Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(u),
		Err(e) => Err(e),
	}
}

async fn _unique_name(mut url: UrlBuf, append: bool) -> io::Result<UrlBuf> {
	let Some(stem) = url.stem().map(|s| s.to_owned()) else {
		return Err(io::Error::new(io::ErrorKind::InvalidInput, "empty file stem"));
	};

	let dot_ext = match url.ext() {
		Some(e) => {
			let mut s = StrandBuf::with_capacity(url.kind(), e.len() + 1);
			s.push_str(".");
			s.try_push(e)?;
			s
		}
		None => StrandBuf::default(),
	};

	let mut name = StrandBuf::with_capacity(url.kind(), stem.len() + dot_ext.len() + 5);
	for i in 1u64.. {
		name.clear();
		name.try_push(&stem)?;

		if append {
			name.try_push(&dot_ext)?;
			name.push_str(format!("_{i}"));
		} else {
			name.push_str(format!("_{i}"));
			name.try_push(&dot_ext)?;
		}

		url.try_set_name(&name)?;
		ok_or_not_found!(provider::symlink_metadata(&url).await, break);
	}

	Ok(url)
}

pub async fn unique_file(u: UrlBuf, is_dir: bool) -> io::Result<UrlBuf> {
	let result = if is_dir {
		provider::create_dir(&u).await
	} else {
		provider::create_new(&u).await.map(|_| ())
	};

	match result {
		Ok(()) => Ok(u),
		Err(e) if e.kind() == io::ErrorKind::AlreadyExists => _unique_file(u, is_dir).await,
		Err(e) => Err(e),
	}
}

async fn _unique_file(mut url: UrlBuf, is_dir: bool) -> io::Result<UrlBuf> {
	let Some(stem) = url.stem().map(|s| s.to_owned()) else {
		return Err(io::Error::new(io::ErrorKind::InvalidInput, "empty file stem"));
	};

	let dot_ext = match url.ext() {
		Some(e) => {
			let mut s = StrandBuf::with_capacity(url.kind(), e.len() + 1);
			s.push_str(".");
			s.try_push(e)?;
			s
		}
		None => StrandBuf::default(),
	};

	let mut name = StrandBuf::with_capacity(url.kind(), stem.len() + dot_ext.len() + 5);
	for i in 1u64.. {
		name.clear();
		name.try_push(&stem)?;

		if is_dir {
			name.try_push(&dot_ext)?;
			name.push_str(format!("_{i}"));
		} else {
			name.push_str(format!("_{i}"));
			name.try_push(&dot_ext)?;
		}

		url.try_set_name(&name)?;
		let result = if is_dir {
			provider::create_dir(&url).await
		} else {
			provider::create_new(&url).await.map(|_| ())
		};

		match result {
			Ok(()) => break,
			Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
			Err(e) => Err(e)?,
		};
	}

	Ok(url)
}
