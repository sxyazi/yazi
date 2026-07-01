use anyhow::anyhow;
use yazi_macro::impl_data_any;
use yazi_shared::data::Data;

use crate::file::File;

impl_data_any!(File, from_into_lua = inherit);

impl TryFrom<Data> for File {
	type Error = anyhow::Error;

	fn try_from(value: Data) -> Result<Self, Self::Error> {
		value.into_any::<Self>().ok_or_else(|| anyhow!("not a File"))
	}
}

impl TryFrom<&Data> for File {
	type Error = anyhow::Error;

	fn try_from(value: &Data) -> Result<Self, Self::Error> {
		value.as_any::<Self>().cloned().ok_or_else(|| anyhow!("not a File"))
	}
}
