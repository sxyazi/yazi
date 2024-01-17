pub const MIME_DIR: &str = "inode/directory";

pub fn mime_valid(b: &[u8]) -> bool {
	let parts = b.split(|&b| b == b'/').collect::<Vec<_>>();
	if parts.len() != 2 || parts[1].is_empty() {
		return false;
	}

	matches!(
		parts[0],
		b"application"
			| b"audio"
			| b"example"
			| b"font"
			| b"image"
			| b"inode"
			| b"message"
			| b"model"
			| b"multipart"
			| b"text"
			| b"video"
	)
}
