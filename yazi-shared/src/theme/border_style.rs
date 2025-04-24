use super::Style;
use ratatui::widgets::BorderType;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub struct BorderStyle {
	#[serde(flatten)]
	pub style: Style,
	pub r#type: BorderType,
}
