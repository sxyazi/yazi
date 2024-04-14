use std::{mem, ops::ControlFlow};

use yazi_shared::{event::Cmd, render};

use crate::completion::Completion;

const LIMIT: usize = 30;

pub struct Opt {
	cache:      Vec<String>,
	cache_name: String,
	word:       String,
	ticket:     usize,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			cache:      mem::take(&mut c.args),
			cache_name: c.take_name("cache-name").unwrap_or_default(),
			word:       c.take_name("word").unwrap_or_default(),
			ticket:     c.take_name("ticket").and_then(|v| v.parse().ok()).unwrap_or(0),
		}
	}
}

impl Completion {
	fn match_candidates(word: &str, cache: &[String]) -> Vec<String> {
		let flow = cache.iter().try_fold(
			(Vec::with_capacity(LIMIT), Vec::with_capacity(LIMIT)),
			|(mut prefixed, mut fuzzy), s| {
				let start_with_word = if word.chars().any(|c| c.is_uppercase()) {
					s.starts_with(word)
				} else {
					s.to_lowercase().starts_with(word)
				};
				if start_with_word {
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

	pub fn show(&mut self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;
		if self.ticket != opt.ticket {
			return;
		}

		if !opt.cache.is_empty() {
			self.caches.insert(opt.cache_name.to_owned(), opt.cache.clone());
		}
		let Some(cache) = self.caches.get(&opt.cache_name) else {
			return;
		};

		self.ticket = opt.ticket;
		self.cands = Self::match_candidates(&opt.word, cache);
		if self.cands.is_empty() {
			return render!(mem::replace(&mut self.visible, false));
		}

		self.offset = 0;
		self.cursor = 0;
		self.visible = true;
		render!();
	}
}
