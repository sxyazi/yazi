use std::path::PathBuf;

pub struct Sftp {
	pub host:     String,
	pub user:     String,
	pub port:     u16,
	pub password: Option<String>,
	pub key_file: Option<PathBuf>,
}
