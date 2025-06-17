use anyhow::bail;

#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Scheme {
	#[default]
	Regular,
	Search,
	SearchItem,
	Archive,
}

impl TryFrom<&[u8]> for Scheme {
	type Error = anyhow::Error;

	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		Ok(match value {
			b"regular" => Scheme::Regular,
			b"search" => Scheme::Search,
			b"archive" => Scheme::Archive,
			_ => bail!("Unknown URL scheme: {}", String::from_utf8_lossy(value)),
		})
	}
}
