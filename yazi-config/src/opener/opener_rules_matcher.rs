use std::{mem, sync::Arc};

use hashbrown::{HashMap, hash_map};

use crate::opener::{Opener, OpenerRulesArc};

pub struct OpenerRulesMatcher {
	iter:    hash_map::Iter<'static, String, OpenerRulesArc>,
	_opener: Arc<HashMap<String, OpenerRulesArc>>,
}

impl From<&Opener> for OpenerRulesMatcher {
	fn from(opener: &Opener) -> Self {
		let opener = opener.load_full();

		let iter = unsafe {
			mem::transmute::<
				hash_map::Iter<'_, String, OpenerRulesArc>,
				hash_map::Iter<'static, String, OpenerRulesArc>,
			>(opener.iter())
		};

		Self { iter, _opener: opener }
	}
}

impl Iterator for OpenerRulesMatcher {
	type Item = (String, OpenerRulesArc);

	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|(name, rules)| (name.clone(), rules.clone()))
	}
}
