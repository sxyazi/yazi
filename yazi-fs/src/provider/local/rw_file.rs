pub struct RwFile(tokio::fs::File);

impl From<tokio::fs::File> for RwFile {
	fn from(value: tokio::fs::File) -> Self { Self(value) }
}

impl From<RwFile> for crate::provider::RwFile {
	fn from(value: RwFile) -> Self { crate::provider::RwFile::Local(value) }
}

impl From<tokio::fs::File> for crate::provider::RwFile {
	fn from(value: tokio::fs::File) -> Self { RwFile(value).into() }
}

impl RwFile {
	#[inline]
	pub fn reader(self) -> tokio::io::BufReader<tokio::fs::File> { tokio::io::BufReader::new(self.0) }

	#[inline]
	pub async fn reader_sync(self) -> std::io::BufReader<std::fs::File> {
		std::io::BufReader::new(self.0.into_std().await)
	}
}
