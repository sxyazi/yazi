use std::{collections::HashMap, str::FromStr};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Provider {
	pub name:   String,
	#[serde(rename = "type")]
	pub type_:  String,
	pub config: HashMap<String, String>,
}

impl TryFrom<Provider> for opendal::Operator {
	type Error = anyhow::Error;

	fn try_from(value: Provider) -> Result<Self, Self::Error> {
		let scheme = opendal::Scheme::from_str(&value.name)?;
		Ok(opendal::Operator::via_map(scheme, value.config)?)
	}
}
