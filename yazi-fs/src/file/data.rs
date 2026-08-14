use anyhow::{anyhow, bail};
use yazi_macro::impl_data_any;
use yazi_shared::data::Data;

use crate::file::{File, Files};

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

impl TryFrom<Data> for Files {
	type Error = anyhow::Error;

	fn try_from(value: Data) -> Result<Self, Self::Error> {
		let Data::List(files) = value else { bail!("not a list of Files") };
		files.into_iter().map(File::try_from).collect::<Result<_, _>>().map(Self)
	}
}

impl TryFrom<&Data> for Files {
	type Error = anyhow::Error;

	fn try_from(value: &Data) -> Result<Self, Self::Error> {
		let Data::List(files) = value else { bail!("not a list of Files") };
		files.iter().map(File::try_from).collect::<Result<_, _>>().map(Self)
	}
}
