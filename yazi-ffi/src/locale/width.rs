use serde::Deserialize;
use strum::EnumIs;

#[derive(Clone, Copy, Default, Deserialize, EnumIs)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Width {
	#[default]
	#[serde(skip)]
	None,
	Numeric,
	#[serde(rename = "2-digit")]
	TwoDigit,
	System,
}

impl From<usize> for Width {
	fn from(len: usize) -> Self { if len == 2 { Self::TwoDigit } else { Self::Numeric } }
}

impl Width {
	pub(super) fn resolve(self, system: Self) -> Self { if self.is_system() { system } else { self } }

	pub(super) fn push(self, field: char, output: &mut String) {
		let (prefix, field) = match (self, field) {
			(Self::None, _) => return,
			(Self::System, _) => unreachable!("system width must be resolved before formatting"),
			(Self::Numeric, 'Y') => ("%", 'Y'),
			(Self::Numeric, field) => ("%-", field),
			(Self::TwoDigit, 'Y') => ("%", 'y'),
			(Self::TwoDigit, field) => ("%", field),
		};

		output.push_str(prefix);
		output.push(field);
	}
}
