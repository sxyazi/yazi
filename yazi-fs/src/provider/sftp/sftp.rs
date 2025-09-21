use std::{io, path::{Path, PathBuf}, sync::Arc};

use yazi_sftp::fs::{Attrs, Flags};
use yazi_shared::scheme::SchemeRef;
use yazi_vfs::config::ProviderSftp;

use crate::{cha::Cha, provider::{FileBuilder, Provider}};

#[derive(Clone, Copy)]
pub struct Sftp {
	name:   &'static str,
	config: &'static ProviderSftp,
}

impl From<(&'static str, &'static ProviderSftp)> for Sftp {
	fn from((name, config): (&'static str, &'static ProviderSftp)) -> Self { Self { name, config } }
}

impl Provider for Sftp {
	type File = yazi_sftp::fs::File;
	type Gate = super::Gate;
	type ReadDir = super::ReadDir;

	fn cache<P>(&self, _: P) -> Option<PathBuf>
	where
		P: AsRef<Path>,
	{
		todo!()
	}

	async fn canonicalize<P>(&self, path: P) -> io::Result<PathBuf>
	where
		P: AsRef<Path>,
	{
		Ok(self.op().await?.realpath(&path).await?)
	}

	async fn copy<P, Q>(&self, from: P, to: Q, cha: Cha) -> io::Result<u64>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		let attrs = Attrs::from(cha);

		let op = self.op().await?;
		let mut from = op.open(&from, Flags::READ, Attrs::default()).await?;
		let mut to = op.open(&to, Flags::WRITE | Flags::CREATE | Flags::TRUNCATE, attrs).await?;

		tokio::io::copy(&mut from, &mut to).await
	}

	async fn create_dir<P>(&self, path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		Ok(self.op().await?.mkdir(&path, Attrs::default()).await?)
	}

	async fn gate(&self) -> io::Result<Self::Gate> {
		super::Gate::new(SchemeRef::Sftp(self.name)).await
	}

	async fn hard_link<P, Q>(&self, original: P, link: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		Ok(self.op().await?.hardlink(&original, &link).await?)
	}

	async fn metadata<P>(&self, path: P) -> io::Result<Cha>
	where
		P: AsRef<Path>,
	{
		let path = path.as_ref();
		let attrs = self.op().await?.stat(path).await?;
		(path.file_name().unwrap_or_default(), &attrs).try_into()
	}

	async fn read_dir<P>(&self, path: P) -> io::Result<Self::ReadDir>
	where
		P: AsRef<Path>,
	{
		Ok(super::ReadDir(self.op().await?.read_dir(&path).await?))
	}

	async fn read_link<P>(&self, path: P) -> io::Result<PathBuf>
	where
		P: AsRef<Path>,
	{
		Ok(self.op().await?.readlink(&path).await?)
	}

	async fn remove_dir<P>(&self, path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		Ok(self.op().await?.rmdir(&path).await?)
	}

	async fn remove_file<P>(&self, path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		Ok(self.op().await?.remove(&path).await?)
	}

	async fn rename<P, Q>(&self, from: P, to: Q) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		Ok(self.op().await?.rename(&from, &to).await?)
	}

	async fn symlink<P, Q, F>(&self, original: P, link: Q, _is_dir: F) -> io::Result<()>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
		F: AsyncFnOnce() -> io::Result<bool>,
	{
		Ok(self.op().await?.symlink(&original, &link).await?)
	}

	async fn symlink_metadata<P>(&self, path: P) -> io::Result<Cha>
	where
		P: AsRef<Path>,
	{
		let path = path.as_ref();
		let attrs = self.op().await?.lstat(path).await?;
		(path.file_name().unwrap_or_default(), &attrs).try_into()
	}

	async fn trash<P>(&self, _path: P) -> io::Result<()>
	where
		P: AsRef<Path>,
	{
		Err(io::Error::new(io::ErrorKind::Unsupported, "Trash not supported"))
	}
}

impl Sftp {
	pub(super) async fn op(self) -> io::Result<deadpool::managed::Object<Sftp>> {
		use deadpool::managed::PoolError;

		let pool = *super::CONN.lock().entry(self.config).or_insert_with(|| {
			Box::leak(Box::new(deadpool::managed::Pool::builder(self).build().unwrap()))
		});

		pool.get().await.map_err(|e| match e {
			PoolError::Timeout(_) => io::Error::new(io::ErrorKind::TimedOut, e.to_string()),
			PoolError::Backend(e) => e,
			PoolError::Closed | PoolError::NoRuntimeSpecified | PoolError::PostCreateHook(_) => {
				io::Error::other(e.to_string())
			}
		})
	}
}

impl russh::client::Handler for Sftp {
	type Error = russh::Error;

	async fn check_server_key(
		&mut self,
		_server_public_key: &russh::keys::PublicKey,
	) -> Result<bool, Self::Error> {
		Ok(true)
	}
}

impl deadpool::managed::Manager for Sftp {
	type Error = io::Error;
	type Type = yazi_sftp::Operator;

	// FIXME: remove the hardcoded test values
	async fn create(&self) -> Result<Self::Type, Self::Error> {
		todo!()
		// async fn inner(sftp: Sftp) -> anyhow::Result<yazi_sftp::Operator> {
		// 	let config = Arc::new(russh::client::Config::default());
		// 	let mut session = russh::client::connect(config, ("127.0.0.1", 22),
		// sftp).await?;

		// 	let mut agent = russh::keys::agent::client::AgentClient::connect_uds(
		// 		"/Users/ika/Library/Group
		// Containers/2BUA8C4S2C.com.1password/t/agent.sock", 	)
		// 	.await?;

		// 	let mut keys = agent.request_identities().await?;
		// 	if !session
		// 		.authenticate_publickey_with("root", keys.remove(0), None, &mut agent)
		// 		.await?
		// 		.success()
		// 	{
		// 		panic!("auth failed");
		// 	}

		// 	let channel = session.channel_open_session().await?;
		// 	channel.request_subsystem(true, "sftp").await?;

		// 	let mut op = yazi_sftp::Operator::make(channel.into_stream());
		// 	op.init().await?;
		// 	Ok(op)
		// }

		// inner(*self).await.map_err(|e| io::Error::other(e.to_string()))
	}

	async fn recycle(
		&self,
		obj: &mut Self::Type,
		metrics: &deadpool::managed::Metrics,
	) -> deadpool::managed::RecycleResult<Self::Error> {
		// FIXME
		Ok(())
	}
}
