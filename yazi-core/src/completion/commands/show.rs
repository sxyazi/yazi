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
		let candidates = cache.iter().try_fold(
			(Vec::with_capacity(candidate_size), Vec::with_capacity(candidate_size)),
			|(mut prefix_cand, mut fuzzy_cand), s| {
				if s.starts_with(opt.word) {
					if s != opt.word {
						prefix_cand.push(s.to_owned());
						if prefix_cand.len() >= candidate_size {
							return ControlFlow::Break((prefix_cand, fuzzy_cand));
						}
					}
				} else if s.contains(opt.word) && fuzzy_cand.len() < candidate_size - prefix_cand.len() {
					// here we don't break the control flow, since we want more exact matching.
					fuzzy_cand.push(s.to_owned())
				}
				ControlFlow::Continue((prefix_cand, fuzzy_cand))
			},
		);

		self.ticket = opt.ticket;
		self.cands = {
			let (mut prefix_cand, fuzzy_cand) = match candidates {
				ControlFlow::Continue(v) => v,
				ControlFlow::Break(v) => v,
			};
			if prefix_cand.len() < candidate_size {
				prefix_cand.extend(fuzzy_cand.into_iter().take(candidate_size - prefix_cand.len()))
			}
			prefix_cand
		};
		if self.cands.is_empty() {
			return mem::replace(&mut self.visible, false);
		}

		self.offset = 0;
		self.cursor = 0;
		self.visible = true;
		true
	}
}
