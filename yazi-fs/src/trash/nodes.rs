use std::vec;

use mlua::{FromLua, Lua, Value};

use super::TrashNode;

pub struct TrashNodes(Vec<TrashNode>);

impl TrashNodes {
	fn new(mut nodes: Vec<TrashNode>) -> Self {
		nodes.sort_unstable_by_key(|node| node.rel.components().count());

		let mut seen = Vec::<TrashNode>::with_capacity(nodes.len());
		for node in nodes {
			if seen.iter().any(|parent| parent.top == node.top && node.rel.starts_with(&parent.rel)) {
				continue;
			}
			seen.push(node);
		}
		Self(seen)
	}
}

impl FromLua for TrashNodes {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let nodes = Vec::<TrashNode>::from_lua(value, lua)?;
		Ok(Self::new(nodes))
	}
}

impl IntoIterator for TrashNodes {
	type IntoIter = vec::IntoIter<TrashNode>;
	type Item = TrashNode;

	fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}
