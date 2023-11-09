use std::{mem, ops::ControlFlow};

use yazi_config::keymap::Exec;

use crate::completion::Completion;

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
	pub fn show<'a>(&mut self, opt: impl Into<Opt<'a>>) -> bool {
		let opt = opt.into();
		if self.ticket != opt.ticket {
			return false;
		}

		if !opt.cache.is_empty() {
			self.caches.insert(opt.cache_name.to_owned(), opt.cache.clone());
		}
		let Some(cache) = self.caches.get(opt.cache_name) else {
			return false;
		};

		let candidate_size = 30;

		// prioritize those with exact prefix
		let exact_flow = cache.iter().try_fold(Vec::with_capacity(candidate_size), |mut v, s| {
			if s.starts_with(opt.word) && s != opt.word {
				v.push(s.to_owned());
				if v.len() >= candidate_size {
					return ControlFlow::Break(v);
				}
			}
			ControlFlow::Continue(v)
		});
		let mut cand = match exact_flow {
			ControlFlow::Continue(v) => v,
			ControlFlow::Break(v) => v,
		};

		let fuzzy_flow =
			cache.iter().try_fold(Vec::with_capacity(candidate_size - cand.len()), |mut v, s| {
				if s.contains(opt.word) && !s.starts_with(opt.word) && s != opt.word {
					v.push(s.to_owned());
					if v.len() >= candidate_size - cand.len() {
						return ControlFlow::Break(v);
					}
				}
				ControlFlow::Continue(v)
			});
		cand.extend(
			match fuzzy_flow {
				ControlFlow::Continue(v) => v,
				ControlFlow::Break(v) => v,
			}
			.into_iter(),
		);

		self.ticket = opt.ticket;
		self.cands = cand;
		if self.cands.is_empty() {
			return mem::replace(&mut self.visible, false);
		}

		self.offset = 0;
		self.cursor = 0;
		self.visible = true;
		true
	}
}
