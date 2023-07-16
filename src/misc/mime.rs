#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MimeKind {
	Dir,
	JSON,
	Text,
	Image,
	Video,
	Others,
}

impl MimeKind {
	pub fn valid(s: &str) -> bool {
		let parts = s.split('/').collect::<Vec<_>>();
		if parts.len() != 2 {
			return false;
		}

		let b = match parts[0] {
			"application" => true,
			"audio" => true,
			"example" => true,
			"font" => true,
			"image" => true,
			"message" => true,
			"model" => true,
			"multipart" => true,
			"text" => true,
			"video" => true,
			_ => false,
		};
		b && !parts[1].is_empty()
	}

	pub fn new(s: &str) -> Self {
		if s == "inode/directory" {
			Self::Dir
		} else if s == "application/json" {
			Self::JSON
		} else if s.starts_with("text/") || s.ends_with("/xml") {
			Self::Text
		} else if s.starts_with("image/") {
			Self::Image
		} else if s.starts_with("video/") {
			Self::Video
		} else {
			Self::Others
		}
	}
}
