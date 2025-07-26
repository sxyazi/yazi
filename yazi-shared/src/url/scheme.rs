use std::fmt::Display;

use anyhow::{Result, bail};

use crate::BytesExt;

#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Scheme {
	#[default]
	Regular,

	Search,
	SearchItem,

	// TODO: redefine
	Archive,

	Sftp(String),
}

impl Scheme {
	pub(super) fn parse(bytes: &[u8]) -> Result<(Self, usize)> {
		let Some((protocol, rest)) = bytes.split_by_seq(b"://") else {
			return Ok((Self::Regular, 0));
		};

		Ok(match protocol {
			b"regular" => (Scheme::Regular, 10),
			b"search" => (Scheme::Search, 9),
			b"archive" => (Scheme::Archive, 10),
			b"sftp" => {
				let (name, skip) = Self::parse_name(rest)?;
				(Scheme::Sftp(name), 7 + skip)
			}
			_ => bail!("Could not parse scheme from URL: {}", String::from_utf8_lossy(bytes)),
		})
	}

	fn parse_name(bytes: &[u8]) -> Result<(String, usize)> {
		let name: Vec<u8> = bytes.iter().copied().take_while(|&b| b != b'/').collect();
		if name.is_empty() {
			bail!("Scheme name cannot be empty");
		} else if !name.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'-') {
			bail!("Scheme name can only contain alphanumeric characters and dashes");
		}

		let len = name.len();
		let slash = bytes.get(len).is_some_and(|&b| b == b'/') as usize;
		Ok((String::from_utf8(name)?, len + slash))
	}
}

impl Display for Scheme {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Scheme::Regular => write!(f, "regular://"),
			Scheme::Search => write!(f, "search://"),
			Scheme::SearchItem => write!(f, "search_item://"),
			Scheme::Archive => write!(f, "archive://"),
			Scheme::Sftp(name) => write!(f, "sftp://{name}/"),
		}
	}
}
