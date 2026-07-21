use std::{io, path::Path, str};

use yazi_fs::{FsAuth, FsHash128, FsUrl, cha::Cha, engine::{Engine, local::Local}};
use yazi_shared::{strand::{AsStrand, StrandCow}, url::{AsUrl, Url, UrlBuf}};

pub struct Stamp(Vec<u8>);

impl Stamp {
	const SIG_LEN: usize = 26;

	pub async fn read<U>(url: U) -> io::Result<Self>
	where
		U: AsUrl,
	{
		let path =
			url.as_url().stamp_entry().ok_or_else(|| io::Error::other("Cannot determine cache stamp"))?;

		Self::read_at(&path).await
	}

	async fn read_at(path: &Path) -> io::Result<Self> {
		let data = Local::regular(path)
			.read()
			.await
			.map_err(|e| io::Error::new(e.kind(), format!("Cannot read cache stamp: {e}")))?;

		Self::try_from(data)
			.map_err(|e| io::Error::new(e.kind(), format!("Cannot parse cache stamp: {e}")))
	}

	pub async fn resolve<U, S>(dir: U, key: S) -> io::Result<UrlBuf>
	where
		U: AsUrl,
		S: AsStrand,
	{
		let dir = dir.as_url();
		let key = key.as_strand();

		let mut path =
			dir.auth().stamp_root().ok_or_else(|| io::Error::other("Cannot determine stamp root"))?;
		path.push(key.as_os()?);

		let stamp = Self::read_at(&path).await?;
		let name = StrandCow::with(dir.kind(), stamp.name()).map_err(io::Error::other)?;

		let url = dir.try_join(name)?;
		if url.hash_u128_str(&mut [0; Self::SIG_LEN]).as_bytes() != key.encoded_bytes() {
			return Err(io::Error::new(io::ErrorKind::InvalidData, "Cache stamp does not match entry"));
		}

		Ok(url)
	}

	pub async fn write(cha: Cha, url: Url<'_>) -> io::Result<()> {
		let path = url.stamp_entry().ok_or_else(|| io::Error::other("Cannot determine cache stamp"))?;
		let data = Self::encode(cha, url)?;

		Local::regular(&path)
			.write(data)
			.await
			.map_err(|e| io::Error::new(e.kind(), format!("Cannot write cache stamp: {e}")))
	}

	fn encode(cha: Cha, url: Url) -> io::Result<Vec<u8>> {
		let name = url.name().ok_or_else(|| io::Error::other("URL has no filename"))?;

		let mut buf = Vec::with_capacity(Self::SIG_LEN + name.len());
		buf.extend_from_slice(cha.hash_u128_str(&mut [0; Self::SIG_LEN]).as_bytes());
		buf.extend_from_slice(name.encoded_bytes());

		Ok(buf)
	}

	pub fn validate(&self, cha: Cha, url: Url) -> io::Result<()> {
		let name = url.name().ok_or_else(|| io::Error::other("URL has no filename"))?;
		if self.name() != name.encoded_bytes() {
			return Err(io::Error::new(io::ErrorKind::InvalidData, "Cache stamp does not match target"));
		}

		if self.sig() != cha.hash_u128_str(&mut [0; Self::SIG_LEN]) {
			return Err(io::Error::new(
				io::ErrorKind::InvalidData,
				"Remote file has changed since last download",
			));
		}
		Ok(())
	}

	#[inline]
	pub fn sig(&self) -> &str { unsafe { str::from_utf8_unchecked(&self.0[..Self::SIG_LEN]) } }

	#[inline]
	pub fn name(&self) -> &[u8] { &self.0[Self::SIG_LEN..] }
}

impl TryFrom<Vec<u8>> for Stamp {
	type Error = io::Error;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		let (sig, _) = value
			.split_at_checked(Self::SIG_LEN)
			.filter(|(_, n)| !n.is_empty())
			.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid cache stamp"))?;

		str::from_utf8(sig).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
		Ok(Self(value))
	}
}
