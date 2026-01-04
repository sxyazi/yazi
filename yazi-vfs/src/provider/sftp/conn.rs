use std::{io, sync::Arc, time::Duration};

use russh::keys::PrivateKeyWithHashAlg;
use yazi_config::vfs::ServiceSftp;
use yazi_fs::provider::local::Local;

#[derive(Clone, Copy)]
pub(super) struct Conn {
	pub(super) name:   &'static str,
	pub(super) config: &'static ServiceSftp,
}

impl russh::client::Handler for Conn {
	type Error = russh::Error;

	async fn check_server_key(
		&mut self,
		_server_public_key: &russh::keys::PublicKey,
	) -> Result<bool, Self::Error> {
		Ok(true)
	}
}

impl deadpool::managed::Manager for Conn {
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

impl Conn {
	pub(super) async fn roll(self) -> io::Result<deadpool::managed::Object<Self>> {
		use deadpool::managed::PoolError;

		let pool = *super::CONN.lock().entry(self.config).or_insert_with(|| {
			Box::leak(Box::new(
				deadpool::managed::Pool::builder(self)
					.runtime(deadpool::Runtime::Tokio1)
					.max_size(8)
					.create_timeout(Some(Duration::from_secs(45)))
					.build()
					.unwrap(),
			))
		});

		pool.get().await.map_err(|e| match e {
			PoolError::Timeout(_) => io::Error::new(io::ErrorKind::TimedOut, e.to_string()),
			PoolError::Backend(e) => e,
			PoolError::Closed | PoolError::NoRuntimeSpecified | PoolError::PostCreateHook(_) => {
				io::Error::other(e.to_string())
			}
		})
	}

	async fn connect(self) -> Result<russh::Channel<russh::client::Msg>, russh::Error> {
		let pref = Arc::new(russh::client::Config {
			inactivity_timeout: Some(std::time::Duration::from_secs(60)),
			keepalive_interval: Some(std::time::Duration::from_secs(10)),
			..Default::default()
		});

		let session = if self.config.password.is_some() {
			self.connect_by_password(pref).await
		} else if !self.config.key_file.as_os_str().is_empty() {
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
		let key_file = &self.config.key_file;
		if key_file.as_os_str().is_empty() {
			return Err(russh::Error::InvalidConfig("Key file not provided".to_owned()));
		};

		let key = Local::regular(key_file)
			.read_to_string()
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
		let identity_agent = &self.config.identity_agent;
		if identity_agent.as_os_str().is_empty() {
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
			let hash_alg = session.best_supported_rsa_hash().await?.flatten();
			match session.authenticate_publickey_with(&self.config.user, key, hash_alg, &mut agent).await
			{
				Ok(result) if result.success() => return Ok(session),
				Ok(result) => tracing::debug!("Identity agent authentication failed: {result:?}"),
				Err(e) => tracing::error!("Identity agent authentication error: {e}"),
			}
		}

		Err(russh::Error::InvalidConfig("Public key authentication via agent failed".to_owned()))
	}
}
