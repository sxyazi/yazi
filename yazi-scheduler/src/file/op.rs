use yazi_fs::cha::Cha;
use yazi_shared::url::Url;

#[derive(Debug)]
pub enum FileOp {
	Paste(FileOpPaste),
	Link(FileOpLink),
	Hardlink(FileOpHardlink),
	Delete(FileOpDelete),
	Trash(FileOpTrash),
}

impl FileOp {
	pub fn id(&self) -> usize {
		match self {
			Self::Paste(op) => op.id,
			Self::Link(op) => op.id,
			Self::Hardlink(op) => op.id,
			Self::Delete(op) => op.id,
			Self::Trash(op) => op.id,
		}
	}
}

// --- Paste
#[derive(Clone, Debug)]
pub struct FileOpPaste {
	pub id:     usize,
	pub from:   Url,
	pub to:     Url,
	pub cha:    Option<Cha>,
	pub cut:    bool,
	pub follow: bool,
	pub retry:  u8,
}

impl FileOpPaste {
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
pub struct FileOpLink {
	pub id:       usize,
	pub from:     Url,
	pub to:       Url,
	pub cha:      Option<Cha>,
	pub resolve:  bool,
	pub relative: bool,
	pub delete:   bool,
}

impl From<FileOpPaste> for FileOpLink {
	fn from(value: FileOpPaste) -> Self {
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
pub struct FileOpHardlink {
	pub id:     usize,
	pub from:   Url,
	pub to:     Url,
	pub cha:    Option<Cha>,
	pub follow: bool,
}

impl FileOpHardlink {
	pub(super) fn spawn(&self, from: Url, to: Url, cha: Cha) -> Self {
		Self { id: self.id, from, to, cha: Some(cha), follow: self.follow }
	}
}

// --- Delete
#[derive(Clone, Debug)]
pub struct FileOpDelete {
	pub id:     usize,
	pub target: Url,
	pub length: u64,
}

// --- Trash
#[derive(Clone, Debug)]
pub struct FileOpTrash {
	pub id:     usize,
	pub target: Url,
	pub length: u64,
}
