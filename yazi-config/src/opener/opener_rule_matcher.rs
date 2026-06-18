use std::sync::Arc;

use mlua::{ExternalError, FromLua, IntoLua, Lua, Table, Value};
use yazi_binding::Iter;
use yazi_shared::id::Id;

use crate::opener::{OpenerRule, OpenerRuleArc, OpenerRulesArc};

#[derive(Default)]
pub struct OpenerRuleMatcher {
	pub rules:  Arc<Vec<OpenerRuleArc>>,
	pub id:     Id,
	pub all:    bool,
	pub offset: usize,
}

impl From<&OpenerRulesArc> for OpenerRuleMatcher {
	fn from(rules: &OpenerRulesArc) -> Self {
		Self { rules: rules.load_full(), all: true, ..Default::default() }
	}
}

impl OpenerRuleMatcher {
	pub fn matches(&self, rule: &OpenerRule) -> bool {
		if self.all {
			true
		} else if self.id != Id::ZERO {
			rule.id == self.id
		} else {
			false
		}
	}
}

impl Iterator for OpenerRuleMatcher {
	type Item = OpenerRuleArc;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(rule) = self.rules.get(self.offset) {
			self.offset += 1;
			if self.matches(rule) {
				return Some(rule.clone());
			}
		}
		None
	}
}

impl TryFrom<Table> for OpenerRuleMatcher {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		let id: Id = t.raw_get("id").unwrap_or_default();

		Ok(Self { id, ..Default::default() })
	}
}

impl FromLua for OpenerRuleMatcher {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(t) => t.try_into(),
			_ => Err("expected a table of OpenerRuleMatcher".into_lua_err()),
		}
	}
}

impl IntoLua for OpenerRuleMatcher {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { Iter::new(self, None).into_lua(lua) }
}
