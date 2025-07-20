use serde::Deserialize;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Priority {
	Low    = 0,
	#[default]
	Normal = 1,
	High   = 2,
}
