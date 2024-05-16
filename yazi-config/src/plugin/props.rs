use super::PluginRule;
use crate::Priority;

#[derive(Debug, Clone)]
pub struct PluginProps {
	pub id:    u8,
	pub name:  String,
	pub multi: bool,
	pub prio:  Priority,
}

impl From<&PluginRule> for PluginProps {
	fn from(rule: &PluginRule) -> Self {
		Self { id: rule.id, name: rule.run.name.to_owned(), multi: rule.multi, prio: rule.prio }
	}
}
