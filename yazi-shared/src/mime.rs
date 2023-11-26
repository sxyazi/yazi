pub const MIME_DIR: &str = "inode/directory";

pub fn mime_valid(s: &str) -> bool {
	let parts = s.split('/').collect::<Vec<_>>();
	if parts.len() != 2 || parts[1].is_empty() {
		return false;
	}

	matches!(
		parts[0],
		"application"
			| "audio"
			| "example"
			| "font"
			| "image"
			| "inode"
			| "message"
			| "model"
			| "multipart"
			| "text"
			| "video"
	)
}
