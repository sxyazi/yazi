use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Flavor {
	#[serde(rename = "use")]
	pub use_: String,
}

impl Flavor {
	pub fn parse_use(s: &str) -> Option<String> {
		#[derive(Deserialize)]
		struct Outer {
			flavor: Inner,
		}
		#[derive(Deserialize)]
		struct Inner {
			#[serde(rename = "use")]
			pub use_: String,
		}

		toml::from_str::<Outer>(s).ok().map(|o| o.flavor.use_).filter(|s| !s.is_empty())
	}
}
