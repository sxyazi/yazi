use strum::IntoStaticStr;
use yazi_shared::id::Id;

#[derive(Debug, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum InputEvent {
	Submit(String),
	Cancel(String),

	Type(String),
	Trigger(String, Option<Id>),
}

impl InputEvent {
	pub fn is_submit(&self) -> bool { matches!(self, Self::Submit(_)) }

	pub fn value(&self) -> &str {
		match self {
			Self::Submit(v) | Self::Cancel(v) | Self::Type(v) | Self::Trigger(v, _) => v.as_str(),
		}
	}

	pub fn map<T, F>(self, f: F) -> Option<T>
	where
		F: FnOnce(String) -> T,
	{
		match self {
			Self::Submit(v) => Some(f(v)),
			_ => None,
		}
	}
}
