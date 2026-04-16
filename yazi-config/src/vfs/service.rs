use serde::{Deserialize, Serialize};

use crate::vfs::ServiceSftp;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Service {
	Sftp(ServiceSftp),
}

impl TryFrom<&'static Service> for &'static ServiceSftp {
	type Error = &'static str;

	fn try_from(value: &'static Service) -> Result<Self, Self::Error> {
		match value {
			Service::Sftp(p) => Ok(p),
		}
	}
}
