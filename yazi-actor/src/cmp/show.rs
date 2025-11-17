use std::{mem, ops::ControlFlow};

use anyhow::Result;
use yazi_macro::{render, succ};
use yazi_parser::cmp::{CmpItem, ShowOpt};
use yazi_shared::{data::Data, path::{AsPathDyn, PathDyn, PathLike}, strand::{AsStrand, StrandLike}};

use crate::{Actor, Ctx};

const LIMIT: usize = 30;

pub struct Show;

impl Actor for Show {
	type Options = ShowOpt;

	const NAME: &str = "show";

	fn act(cx: &mut Ctx, opt: Self::Options) -> Result<Data> {
		let cmp = &mut cx.cmp;
		if cmp.ticket != opt.ticket {
			succ!();
		}

		if !opt.cache.is_empty() {
			cmp.caches.insert(opt.cache_name.clone(), opt.cache);
		}
		let Some(cache) = cmp.caches.get(&opt.cache_name) else {
			succ!();
		};

		cmp.ticket = opt.ticket;
		cmp.cands = Self::match_candidates(opt.word.as_path_dyn(), cache);
		if cmp.cands.is_empty() {
			succ!(render!(mem::replace(&mut cmp.visible, false)));
		}

		cmp.offset = 0;
		cmp.cursor = 0;
		cmp.visible = true;
		succ!(render!());
	}
}

impl Show {
	fn match_candidates(word: PathDyn, cache: &[CmpItem]) -> Vec<CmpItem> {
		let smart = !word.encoded_bytes().iter().any(|&b| b.is_ascii_uppercase());

		let flow = cache.iter().try_fold((Vec::new(), Vec::new()), |(mut exact, mut fuzzy), item| {
			let name = item.name.as_strand();
			let starts_with =
				if smart { name.eq_ignore_ascii_case(word) } else { name.starts_with(word) };

			if starts_with {
				exact.push(item);
				if exact.len() >= LIMIT {
					return ControlFlow::Break((exact, fuzzy));
				}
			} else if fuzzy.len() < LIMIT - exact.len() && name.contains(word) {
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
