use yazi_fs::cha::Cha;
use yazi_shared::url::Url;

#[derive(Debug)]
pub enum FileIn {
	Paste(FileInPaste),
	Link(FileInLink),
	Hardlink(FileInHardlink),
	Delete(FileInDelete),
	Trash(FileInTrash),
}

impl FileIn {
	pub fn id(&self) -> usize {
		match self {
			Self::Paste(r#in) => r#in.id,
			Self::Link(r#in) => r#in.id,
			Self::Hardlink(r#in) => r#in.id,
			Self::Delete(r#in) => r#in.id,
			Self::Trash(r#in) => r#in.id,
		}
	}
}

// --- Paste
#[derive(Clone, Debug)]
pub struct FileInPaste {
	pub id:     usize,
	pub from:   Url,
	pub to:     Url,
	pub cha:    Option<Cha>,
	pub cut:    bool,
	pub follow: bool,
	pub retry:  u8,
}

impl FileInPaste {
	pub(super) fn spawn(&self, from: Url, to: Url, cha: Cha) -> Self {
		Self {
			id: self.id,
			from,
			to,
			cha: Some(cha),
			cut: self.cut,
			follow: self.follow,
			retry: self.retry,
		}
	}
}

// --- Link
#[derive(Clone, Debug)]
pub struct FileInLink {
	pub id:       usize,
	pub from:     Url,
	pub to:       Url,
	pub cha:      Option<Cha>,
	pub resolve:  bool,
	pub relative: bool,
	pub delete:   bool,
}

impl From<FileInPaste> for FileInLink {
	fn from(value: FileInPaste) -> Self {
		Self {
			id:       value.id,
			from:     value.from,
			to:       value.to,
			cha:      value.cha,
			resolve:  true,
			relative: false,
			delete:   value.cut,
		}
	}
}

// --- Hardlink
#[derive(Clone, Debug)]
pub struct FileInHardlink {
	pub id:     usize,
	pub from:   Url,
	pub to:     Url,
	pub cha:    Option<Cha>,
	pub follow: bool,
}

impl FileInHardlink {
	pub(super) fn spawn(&self, from: Url, to: Url, cha: Cha) -> Self {
		Self { id: self.id, from, to, cha: Some(cha), follow: self.follow }
	}
}

// --- Delete
#[derive(Clone, Debug)]
pub struct FileInDelete {
	pub id:     usize,
	pub target: Url,
	pub length: u64,
}

// --- Trash
#[derive(Clone, Debug)]
pub struct FileInTrash {
	pub id:     usize,
	pub target: Url,
	pub length: u64,
}
