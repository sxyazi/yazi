use std::{collections::HashMap, str::FromStr};

use anyhow::{anyhow, Result};
use opendal::Operator;

use crate::RoCell;

/// SCHEMES is the read only cell of schemes that already loaded.
pub static SCHEMES: RoCell<Schemes> = RoCell::new();

/// Schemes carries all the schemes defined in the configuration file.
#[derive(Debug)]
pub struct Schemes(HashMap<String, Operator>);

impl Schemes {
	/// Build schemes from iterator.
	pub fn from_iter(
		iter: impl IntoIterator<Item = (String, String, HashMap<String, String>)>,
	) -> Result<Self> {
		let mut schemes = HashMap::new();
		for (name, typ, config) in iter {
			let scheme = opendal::Scheme::from_str(&typ)?;
			let operator = Operator::via_map(scheme, config)?;
			schemes.insert(name, operator);
		}
		Ok(Self(schemes))
	}

	/// Get operator by scheme name.
	pub fn get(&self, scheme: &str) -> Result<Operator> {
		self.0.get(scheme).cloned().ok_or(anyhow!("storage scheme {scheme} is not configured"))
	}
}
