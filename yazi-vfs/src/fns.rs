use std::io::{self};

use yazi_shared::{strand::{StrandBuf, StrandLike}, url::{AsUrl, UrlBuf, UrlLike}};

use crate::engine;

pub async fn maybe_exists(url: impl AsUrl) -> bool {
	match engine::symlink_metadata(url).await {
		Ok(_) => true,
		Err(e) => e.kind() != io::ErrorKind::NotFound,
	}
}

pub async fn unique_file(u: UrlBuf, is_dir: bool) -> io::Result<UrlBuf> {
	let result =
		if is_dir { engine::create_dir(&u).await } else { engine::create_new(&u).await.map(|_| ()) };

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
			engine::create_dir(&url).await
		} else {
			engine::create_new(&url).await.map(|_| ())
		};

		match result {
			Ok(()) => break,
			Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
			Err(e) => Err(e)?,
		};
	}

	Ok(url)
}
