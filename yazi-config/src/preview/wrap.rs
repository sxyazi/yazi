use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreviewWrap {
	No,
	Yes,
}

impl From<PreviewWrap> for Option<ratatui::widgets::Wrap> {
	fn from(wrap: PreviewWrap) -> Self {
		match wrap {
			PreviewWrap::No => None,
			PreviewWrap::Yes => Some(ratatui::widgets::Wrap { trim: false }),
		}
	}
}

impl From<PreviewWrap> for yazi_binding::elements::Wrap {
	fn from(value: PreviewWrap) -> Self {
		Self(match value {
			PreviewWrap::No => None,
			PreviewWrap::Yes => Some(ratatui::widgets::Wrap { trim: false }),
		})
	}
}
