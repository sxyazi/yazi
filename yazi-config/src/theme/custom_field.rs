use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum CustomField {
	Style(crate::Style),
	String(String),
}

impl From<&CustomField> for CustomField {
	fn from(value: &CustomField) -> Self { value.clone() }
}
