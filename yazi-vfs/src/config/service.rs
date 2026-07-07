use serde::{Deserialize, Serialize};

use crate::config::{ServiceRclone, ServiceSftp};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Service {
	Sftp(ServiceSftp),
	Rclone(ServiceRclone),
}

impl TryFrom<&'static Service> for &'static ServiceSftp {
	type Error = &'static str;

	fn try_from(value: &'static Service) -> Result<Self, Self::Error> {
		match value {
			Service::Sftp(p) => Ok(p),
			_ => Err("expected an SFTP service"),
		}
	}
}

impl TryFrom<&'static Service> for &'static ServiceRclone {
	type Error = &'static str;

	fn try_from(value: &'static Service) -> Result<Self, Self::Error> {
		match value {
			Service::Rclone(p) => Ok(p),
			_ => Err("expected an rclone service"),
		}
	}
}
