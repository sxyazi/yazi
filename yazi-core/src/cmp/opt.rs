use yazi_shared::{Id, path::PathBufDyn, url::UrlBuf};

use crate::cmp::CmpItem;

#[derive(Clone, Debug)]
pub struct CmpOpt {
	pub cache:      Vec<CmpItem>,
	pub cache_name: UrlBuf,
	pub word:       PathBufDyn,
	pub ticket:     Id,
}
