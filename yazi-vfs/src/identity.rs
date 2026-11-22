use std::io;

use tokio::try_join;
use yazi_fs::engine::Engine;
use yazi_macro::ok_or_not_found;
use yazi_shared::url::{Url, UrlBuf, UrlLike};

pub(super) struct Identity;

impl Identity {
	pub(super) async fn same_entry<E: Engine>(a: &E, b: Url<'_>) -> io::Result<bool> {
		let b = b.physical();
		if !a.as_url().auth().same_service(b.auth()) {
			return Ok(false);
		}

		let (a, b) = try_join!(Self::resolve_entry::<E>(a.as_url()), Self::resolve_entry::<E>(b))?;
		Ok(a == b)
	}

	pub(super) async fn same_file<E: Engine>(a: &E, b: Url<'_>) -> io::Result<bool> {
		let b = b.physical();
		if !a.as_url().auth().same_service(b.auth()) {
			return Ok(false);
		}

		let b = ok_or_not_found!(E::new(b).await?.canonicalize().await, return Ok(false));
		Ok(a.canonicalize().await? == b)
	}

	async fn resolve_entry<E: Engine>(url: Url<'_>) -> io::Result<UrlBuf> {
		let (parent, name) = url.parent().zip(url.name()).ok_or(io::ErrorKind::InvalidInput)?;
		let parent = E::new(parent).await?.canonicalize().await?;

		Ok(parent.try_join(name)?)
	}
}
