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

#[inline]
pub async fn must_be_dir(url: impl AsUrl) -> bool {
	provider::metadata(url).await.is_ok_and(|m| m.is_dir())
}

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
