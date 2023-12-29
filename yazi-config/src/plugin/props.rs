use super::PluginRule;
use crate::Priority;

#[derive(Debug, Clone)]
pub struct PluginProps {
	pub id:    u8,
	pub cmd:   String,
	pub multi: bool,
	pub prio:  Priority,
}

impl From<&PluginRule> for PluginProps {
	fn from(rule: &PluginRule) -> Self {
		Self { id: rule.id, cmd: rule.exec.cmd.to_owned(), multi: rule.multi, prio: rule.prio }
	}
}
