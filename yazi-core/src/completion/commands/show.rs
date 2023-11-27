use std::{mem, ops::ControlFlow};

use yazi_shared::Exec;

use crate::completion::Completion;

const LIMIT: usize = 30;

pub struct Opt<'a> {
	cache:      &'a Vec<String>,
	cache_name: &'a str,
	word:       &'a str,
	ticket:     usize,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self {
		Self {
			cache:      &e.args,
			cache_name: e.named.get("cache-name").map(|n| n.as_str()).unwrap_or_default(),
			word:       e.named.get("word").map(|w| w.as_str()).unwrap_or_default(),
			ticket:     e.named.get("ticket").and_then(|v| v.parse().ok()).unwrap_or(0),
		}
	}
}

impl Completion {
	fn match_candidates(word: &str, cache: &[String]) -> Vec<String> {
		let flow = cache.iter().try_fold(
			(Vec::with_capacity(LIMIT), Vec::with_capacity(LIMIT)),
			|(mut prefixed, mut fuzzy), s| {
				if s.starts_with(word) {
					if s != word {
						prefixed.push(s);
						if prefixed.len() >= LIMIT {
							return ControlFlow::Break((prefixed, fuzzy));
						}
					}
				} else if fuzzy.len() < LIMIT - prefixed.len() && s.contains(word) {
					// here we don't break the control flow, since we want more exact matching.
					fuzzy.push(s)
				}
				ControlFlow::Continue((prefixed, fuzzy))
			},
		);

		let (mut prefixed, fuzzy) = match flow {
			ControlFlow::Continue(v) => v,
			ControlFlow::Break(v) => v,
		};
		if prefixed.len() < LIMIT {
			prefixed.extend(fuzzy.into_iter().take(LIMIT - prefixed.len()))
		}
		prefixed.into_iter().map(ToOwned::to_owned).collect()
	}

	pub fn show<'a>(&mut self, opt: impl Into<Opt<'a>>) -> bool {
		let opt = opt.into() as Opt;
		if self.ticket != opt.ticket {
			return false;
		}

		if !opt.cache.is_empty() {
			self.caches.insert(opt.cache_name.to_owned(), opt.cache.clone());
		}
		let Some(cache) = self.caches.get(opt.cache_name) else {
			return false;
		};

		self.ticket = opt.ticket;
		self.cands = Self::match_candidates(opt.word, cache);
		if self.cands.is_empty() {
			return mem::replace(&mut self.visible, false);
		}

		self.offset = 0;
		self.cursor = 0;
		self.visible = true;
		true
	}
}
