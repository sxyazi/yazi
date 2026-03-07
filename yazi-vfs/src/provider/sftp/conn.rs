use std::{io, sync::Arc, time::{Duration, SystemTime}};

use russh::keys::PrivateKeyWithHashAlg;
use yazi_config::vfs::ServiceSftp;
use yazi_fs::provider::local::Local;

#[derive(Clone, Copy)]
pub(super) struct Conn {
	pub(super) name:   &'static str,
	pub(super) config: &'static ServiceSftp,
}

macro_rules! cfg_err {
	($($args:tt)*) => {
		russh::Error::InvalidConfig(format!($($args)*))
	};
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
		} else if !self.config.key_file.as_os_str().is_empty()
			&& !self.config.cert_file.as_os_str().is_empty()
		{
			self.connect_by_key_and_cert(pref).await
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
			return Err(cfg_err!("Password not provided"));
		};

		let mut session =
			russh::client::connect(pref, (self.config.host.as_str(), self.config.port), self).await?;

		if session.authenticate_password(&self.config.user, password).await?.success() {
			Ok(session)
		} else {
			Err(cfg_err!("Password authentication failed"))
		}
	}

	async fn connect_by_key(
		self,
		pref: Arc<russh::client::Config>,
	) -> Result<russh::client::Handle<Self>, russh::Error> {
		let key_file = &self.config.key_file;
		if key_file.as_os_str().is_empty() {
			return Err(cfg_err!("Key file not provided"));
		};

		let key = Local::regular(key_file)
			.read_to_string()
			.await
			.map_err(|e| cfg_err!("Failed to read key file: {e}"))?;
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

		if result.success() { Ok(session) } else { Err(cfg_err!("Public key authentication failed")) }
	}

	async fn connect_by_key_and_cert(
		self,
		pref: Arc<russh::client::Config>,
	) -> Result<russh::client::Handle<Self>, russh::Error> {
		let key_file = &self.config.key_file;
		if key_file.as_os_str().is_empty() {
			return Err(cfg_err!("Key file not provided"));
		};

		let cert_file = &self.config.cert_file;
		if cert_file.as_os_str().is_empty() {
			return Err(cfg_err!("Cert file not provided"));
		};

		// Decode the key and cert files
		let key = Local::regular(key_file)
			.read_to_string()
			.await
			.map_err(|e| cfg_err!("Failed to read key file: {e}"))?;
		let key = russh::keys::decode_secret_key(&key, self.config.key_passphrase.as_deref())?;

		let cert = Local::regular(cert_file)
			.read_to_string()
			.await
			.map_err(|e| cfg_err!("Failed to read cert file: {e}"))?;
		let cert = russh::keys::Certificate::from_openssh(&cert)?;

		// Verify the certificate
		if !self.config.no_cert_verify {
			cert
				.verify_signature()
				.map_err(|e| cfg_err!("Certificate signature verification failed: {e}"))?;

			let now: chrono::DateTime<chrono::Local> = SystemTime::now().into();
			let start: chrono::DateTime<chrono::Local> = cert.valid_after_time().into();
			let end: chrono::DateTime<chrono::Local> = cert.valid_before_time().into();
			if now < start || now > end {
				return Err(cfg_err!(
					"Certificate is out of the validity range of '{}' to '{}'",
					start.to_rfc2822(),
					end.to_rfc2822()
				));
			}
		}

		let mut session =
			russh::client::connect(pref, (self.config.host.as_str(), self.config.port), self).await?;

		if session.authenticate_openssh_cert(&self.config.user, Arc::new(key), cert).await?.success() {
			Ok(session)
		} else {
			Err(cfg_err!("Public key with certificate authentication failed"))
		}
	}

	async fn connect_by_agent(
		self,
		pref: Arc<russh::client::Config>,
	) -> Result<russh::client::Handle<Self>, russh::Error> {
		let identity_agent = &self.config.identity_agent;
		if identity_agent.as_os_str().is_empty() {
			return Err(cfg_err!("Identity agent not provided"));
		};

		#[cfg(unix)]
		let mut agent = russh::keys::agent::client::AgentClient::connect_uds(identity_agent).await?;
		#[cfg(windows)]
		let mut agent =
			russh::keys::agent::client::AgentClient::connect_named_pipe(identity_agent).await?;

		let keys = agent.request_identities().await?;
		if keys.is_empty() {
			return Err(cfg_err!("No keys found in SSH agent"));
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

		Err(cfg_err!("Public key authentication via agent failed"))
	}
}
