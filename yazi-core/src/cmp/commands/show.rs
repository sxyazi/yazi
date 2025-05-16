use std::{borrow::Cow, mem, ops::ControlFlow, path::PathBuf};

use yazi_macro::render;
use yazi_proxy::options::CmpItem;
use yazi_shared::{Id, event::{Cmd, CmdCow, Data}, osstr_contains, osstr_starts_with};

use crate::cmp::Cmp;

const LIMIT: usize = 30;

struct Opt {
	cache:      Vec<CmpItem>,
	cache_name: PathBuf,
	word:       Cow<'static, str>,
	ticket:     Id,
}

impl From<CmdCow> for Opt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			cache:      c.take_any("cache").unwrap_or_default(),
			cache_name: c.take_any("cache-name").unwrap_or_default(),
			word:       c.take_str("word").unwrap_or_default(),
			ticket:     c.get("ticket").and_then(Data::as_id).unwrap_or_default(),
		}
	}
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self::from(CmdCow::from(c)) }
}

impl Cmp {
	#[yazi_codegen::command]
	pub fn show(&mut self, opt: Opt) {
		if self.ticket != opt.ticket {
			return;
		}

		if !opt.cache.is_empty() {
			self.caches.insert(opt.cache_name.clone(), opt.cache);
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

	fn match_candidates(word: &str, cache: &[CmpItem]) -> Vec<CmpItem> {
		let smart = !word.bytes().any(|c| c.is_ascii_uppercase());

		let flow = cache.iter().try_fold((Vec::new(), Vec::new()), |(mut exact, mut fuzzy), item| {
			if osstr_starts_with(&item.name, word, smart) {
				exact.push(item);
				if exact.len() >= LIMIT {
					return ControlFlow::Break((exact, fuzzy));
				}
			} else if fuzzy.len() < LIMIT - exact.len() && osstr_contains(&item.name, word) {
				// Here we don't break the control flow, since we want more exact matching.
				fuzzy.push(item)
			}
			ControlFlow::Continue((exact, fuzzy))
		});

		let (exact, fuzzy) = match flow {
			ControlFlow::Continue(v) => v,
			ControlFlow::Break(v) => v,
		};

		let it = fuzzy.into_iter().take(LIMIT - exact.len());
		exact.into_iter().chain(it).cloned().collect()
	}
}
