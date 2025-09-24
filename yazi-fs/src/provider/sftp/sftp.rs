use std::{io, path::{Path, PathBuf}, sync::Arc};

use russh::keys::PrivateKeyWithHashAlg;
use tokio::io::{BufReader, BufWriter};
use yazi_sftp::fs::{Attrs, Flags};
use yazi_shared::{scheme::SchemeRef, url::{Url, UrlBuf, UrlCow}};
use yazi_vfs::config::ProviderSftp;

use crate::{cha::Cha, provider::{DirReader, FileBuilder, FileHolder, Provider, local::Local}};

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

	async fn absolute<'a, U>(&self, url: U) -> io::Result<UrlCow<'a>>
	where
		U: Into<Url<'a>>,
	{
		let url: Url = url.into();
		Ok(if url.is_absolute() {
			url.into()
		} else if let SchemeRef::Sftp(_) = url.scheme {
			UrlBuf { loc: self.canonicalize(url.loc).await?.into(), scheme: url.scheme.into() }.into()
		} else {
			Err(io::Error::new(io::ErrorKind::InvalidInput, "Not an SFTP URL"))?
		})
	}

	async fn canonicalize<P>(&self, path: P) -> io::Result<PathBuf>
	where
		P: AsRef<Path>,
	{
		Ok(self.op().await?.realpath(&path).await?)
	}

	async fn casefold<P>(&self, path: P) -> io::Result<PathBuf>
	where
		P: AsRef<Path>,
	{
		let path = path.as_ref();
		let Some((parent, name)) = path.parent().zip(path.file_name()) else {
			return Ok(path.to_owned());
		};

		if !self.symlink_metadata(path).await?.is_link() {
			return match self.canonicalize(path).await?.file_name() {
				Some(name) => Ok(parent.join(name)),
				None => Err(io::Error::other("Cannot get filename")),
			};
		}

		let mut it = self.read_dir(parent).await?;
		let mut similar = None;
		while let Some(entry) = it.next().await? {
			let s = entry.name();
			if !s.eq_ignore_ascii_case(name) {
				continue;
			} else if s == name {
				return Ok(entry.path());
			} else if similar.is_none() {
				similar = Some(s.into_owned());
			}
		}

		similar.map(|n| parent.join(n)).ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
	}

	async fn copy<P, Q>(&self, from: P, to: Q, cha: Cha) -> io::Result<u64>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		let attrs = Attrs::from(cha);

		let op = self.op().await?;
		let from = op.open(&from, Flags::READ, &Attrs::default()).await?;
		let to = op.open(&to, Flags::WRITE | Flags::CREATE | Flags::TRUNCATE, &attrs).await?;

		let mut reader = BufReader::with_capacity(524288, from);
		let mut writer = BufWriter::with_capacity(524288, to);

		let written = tokio::io::copy(&mut reader, &mut writer).await?;
		writer.into_inner().fsetstat(&attrs).await.ok();

		Ok(written)
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
			Box::leak(Box::new(deadpool::managed::Pool::builder(self).max_size(5).build().unwrap()))
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

	async fn create(&self) -> Result<Self::Type, Self::Error> {
		let channel = self.connect().await.map_err(|e| {
			io::Error::other(format!("Failed to connect to SFTP server `{}`: {e}", self.name))
		})?;

		let mut op = yazi_sftp::Operator::make(channel.into_stream());
		op.init().await?;
		Ok(op)
	}

	async fn recycle(
		&self,
		obj: &mut Self::Type,
		_metrics: &deadpool::managed::Metrics,
	) -> deadpool::managed::RecycleResult<Self::Error> {
		if obj.is_closed() {
			Err(deadpool::managed::RecycleError::Message("Channel closed".into()))
		} else {
			Ok(())
		}
	}
}

impl Sftp {
	async fn connect(self) -> Result<russh::Channel<russh::client::Msg>, russh::Error> {
		let pref = Arc::new(russh::client::Config {
			inactivity_timeout: Some(std::time::Duration::from_secs(30)),
			keepalive_interval: Some(std::time::Duration::from_secs(10)),
			nodelay: true,
			..Default::default()
		});

		let session = if self.config.password.is_some() {
			self.connect_by_password(pref).await
		} else if self.config.key_file.is_some() {
			self.connect_by_key(pref).await
		} else {
			self.connect_by_agent(pref).await
		}?;

		let channel = session.channel_open_session().await?;
		channel.request_subsystem(true, "sftp").await?;
		Ok(channel)
	}

	async fn connect_by_password(
		self,
		pref: Arc<russh::client::Config>,
	) -> Result<russh::client::Handle<Self>, russh::Error> {
		let Some(password) = &self.config.password else {
			return Err(russh::Error::InvalidConfig("Password not provided".to_owned()));
		};

		let mut session =
			russh::client::connect(pref, (self.config.host.as_str(), self.config.port), self).await?;

		if session.authenticate_password(&self.config.user, password).await?.success() {
			Ok(session)
		} else {
			Err(russh::Error::InvalidConfig("Password authentication failed".to_owned()))
		}
	}

	async fn connect_by_key(
		self,
		pref: Arc<russh::client::Config>,
	) -> Result<russh::client::Handle<Self>, russh::Error> {
		let Some(key_file) = &self.config.key_file else {
			return Err(russh::Error::InvalidConfig("Key file not provided".to_owned()));
		};

		let key = Local
			.read_to_string(key_file)
			.await
			.map_err(|e| russh::Error::InvalidConfig(format!("Failed to read key file: {e}")))?;

		let key = russh::keys::decode_secret_key(&key, self.config.key_passphrase.as_deref())?;

		let mut session =
			russh::client::connect(pref, (self.config.host.as_str(), self.config.port), self).await?;

		let result = session
			.authenticate_publickey(
				&self.config.user,
				PrivateKeyWithHashAlg::new(
					Arc::new(key),
					session.best_supported_rsa_hash().await?.flatten(),
				),
			)
			.await?;

		if result.success() {
			Ok(session)
		} else {
			Err(russh::Error::InvalidConfig("Public key authentication failed".to_owned()))
		}
	}

	async fn connect_by_agent(
		self,
		pref: Arc<russh::client::Config>,
	) -> Result<russh::client::Handle<Self>, russh::Error> {
		let Some(identity_agent) = &self.config.identity_agent else {
			return Err(russh::Error::InvalidConfig("Identity agent not provided".to_owned()));
		};

		#[cfg(unix)]
		let mut agent = russh::keys::agent::client::AgentClient::connect_uds(identity_agent).await?;
		#[cfg(windows)]
		let mut agent =
			russh::keys::agent::client::AgentClient::connect_named_pipe(identity_agent).await?;

		let keys = agent.request_identities().await?;
		if keys.is_empty() {
			return Err(russh::Error::InvalidConfig("No keys found in SSH agent".to_owned()));
		}

		let mut session =
			russh::client::connect(pref, (self.config.host.as_str(), self.config.port), self).await?;

		for key in keys {
			match session.authenticate_publickey_with(&self.config.user, key, None, &mut agent).await {
				Ok(result) if result.success() => return Ok(session),
				Ok(_) => {}
				Err(e) => tracing::error!("Identity agent authentication error: {e}"),
			}
		}

		Err(russh::Error::InvalidConfig("Public key authentication via agent failed".to_owned()))
	}
}
