use std::fs::Metadata;

use yazi_shared::fs::Url;

#[derive(Debug)]
pub enum FileOp {
	Paste(FileOpPaste),
	Link(FileOpLink),
	Delete(FileOpDelete),
	Trash(FileOpTrash),
}

impl FileOp {
	pub fn id(&self) -> usize {
		match self {
			Self::Paste(op) => op.id,
			Self::Link(op) => op.id,
			Self::Delete(op) => op.id,
			Self::Trash(op) => op.id,
		}
	}
}

#[derive(Clone, Debug)]
pub struct FileOpPaste {
	pub id:     usize,
	pub from:   Url,
	pub to:     Url,
	pub meta:   Option<Metadata>,
	pub cut:    bool,
	pub follow: bool,
	pub retry:  u8,
}

impl FileOpPaste {
	pub(super) fn spawn(&self, from: Url, to: Url, meta: Metadata) -> Self {
		Self {
			id: self.id,
			from,
			to,
			meta: Some(meta),
			cut: self.cut,
			follow: self.follow,
			retry: self.retry,
		}
	}
}

#[derive(Clone, Debug)]
pub struct FileOpLink {
	pub id:       usize,
	pub from:     Url,
	pub to:       Url,
	pub meta:     Option<Metadata>,
	pub resolve:  bool,
	pub relative: bool,
	pub delete:   bool,
}

impl From<FileOpPaste> for FileOpLink {
	fn from(value: FileOpPaste) -> Self {
		Self {
			id:       value.id,
			from:     value.from.clone(),
			to:       value.to.clone(),
			meta:     value.meta.clone(),
			resolve:  true,
			relative: false,
			delete:   value.cut,
		}
	}
}

#[derive(Clone, Debug)]
pub struct FileOpDelete {
	pub id:     usize,
	pub target: Url,
	pub length: u64,
}

#[derive(Clone, Debug)]
pub struct FileOpTrash {
	pub id:     usize,
	pub target: Url,
	pub length: u64,
}
