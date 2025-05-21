#[derive(Debug)]
pub enum FileOut {
	Paste(FileOutPaste),
	Link(FileOutLink),
	Hardlink(FileOutHardlink),
	Delete(FileOutDelete),
	Trash(FileOutTrash),
}

#[derive(Debug)]
pub struct FileOutPaste;

#[derive(Debug)]
pub struct FileOutLink;

#[derive(Debug)]
pub struct FileOutHardlink;

#[derive(Debug)]
pub struct FileOutDelete;

#[derive(Debug)]
pub struct FileOutTrash;
