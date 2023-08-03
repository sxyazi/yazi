pub const MIME_DIR: &str = "inode/directory";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MimeKind {
	Dir,
	JSON,
	Text,
	Image,
	Video,
    Pdf,
	Archive,
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
            "pdf" => true,
			"text" => true,
			"video" => true,
			_ => false,
		};
		b && !parts[1].is_empty()
	}

	pub fn new(s: &str) -> Self {
		if s == MIME_DIR {
			Self::Dir
		} else if s == "application/json" {
			Self::JSON
		} else if s == "application/pdf" {
			Self::Pdf
		} else if s.starts_with("text/") || s.ends_with("/xml") {
			Self::Text
		} else if s.starts_with("image/") {
			Self::Image
		} else if s.starts_with("video/") {
			Self::Video
		} else if s == "application/x-bzip"
			|| s == "application/x-bzip2"
			|| s == "application/gzip"
			|| s == "application/vnd.rar"
			|| s == "application/x-tar"
			|| s == "application/zip"
			|| s == "application/x-7z-compressed"
		{
			Self::Archive
		} else {
			Self::Others
		}
	}
}
