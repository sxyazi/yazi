use serde::{Deserialize, Serialize};
use yazi_codegen::DeserializeOver2;

use super::SortBy;

#[derive(Debug, Deserialize, DeserializeOver2, Serialize)]
pub struct Which {
	// Sorting
	pub sort_by:        SortBy,
	pub sort_sensitive: bool,
	pub sort_reverse:   bool,
	pub sort_translit:  bool,
}
