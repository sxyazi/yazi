use serde::Deserialize;

use crate::{auth::AuthArc, wire::Wire};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct View {
	pub source: AuthArc,
	pub data:   Wire,
}
