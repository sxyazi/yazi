use yazi_shared::url::UrlBuf;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CmpKey {
	Authorities,
	Directory(UrlBuf),
	DirectoryWithAuthorities(UrlBuf),
}

impl CmpKey {
	pub fn parent(&self) -> Option<&UrlBuf> {
		match self {
			Self::Authorities => None,
			Self::Directory(u) | Self::DirectoryWithAuthorities(u) => Some(u),
		}
	}
}
