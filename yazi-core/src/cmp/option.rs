use yazi_macro::impl_data_any;
use yazi_shared::{id::Id, path::PathBufDyn, url::UrlBuf};

use crate::cmp::CmpItem;

#[derive(Clone, Debug)]
pub struct CmpOpt {
	pub cache:      Vec<CmpItem>,
	pub cache_name: UrlBuf,
	pub word:       PathBufDyn,
	pub ticket:     Id,
}

impl_data_any!(CmpOpt);
