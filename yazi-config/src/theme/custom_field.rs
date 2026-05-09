use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum CustomField {
	Style(crate::Style),
	String(String),
}

impl From<&Self> for CustomField {
	fn from(value: &Self) -> Self { value.clone() }
}
