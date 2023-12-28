use std::fs::Metadata;

use yazi_shared::fs::Url;

#[derive(Debug)]
pub enum FileOp {
	Paste(FileOpPaste),
	Link(FileOpLink),
	Delete(FileOpDelete),
	Trash(FileOpTrash),
}

#[derive(Clone, Debug)]
pub struct FileOpPaste {
	pub id:     usize,
	pub from:   Url,
	pub to:     Url,
	pub cut:    bool,
	pub follow: bool,
	pub retry:  u8,
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
