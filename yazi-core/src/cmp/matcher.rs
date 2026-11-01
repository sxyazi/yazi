use yazi_shared::strand::{Strand, StrandLike};
use yazi_widgets::cmp::CmpItem;

const LIMIT: usize = 30;

pub(super) struct Matcher<'a> {
	items: &'a [CmpItem],
}

impl<'a> Matcher<'a> {
	pub fn new(items: &'a [CmpItem]) -> Self { Self { items } }

	pub fn matches(&self, word: Strand<'_>) -> impl Iterator<Item = &'a CmpItem> + use<'a> {
		let smart = !word.encoded_bytes().iter().any(|&b| b.is_ascii_uppercase());

		let (mut exact, mut fuzzy) = (vec![], vec![]);
		for item in self.items {
			let starts_with = if smart {
				item.name.starts_with_ignore_ascii_case(word)
			} else {
				item.name.starts_with(word)
			};

			if starts_with {
				exact.push(item);
				if exact.len() >= LIMIT {
					break;
				}
			} else if fuzzy.len() < LIMIT - exact.len() && item.name.contains(word) {
				fuzzy.push(item);
			}
		}

		let it = fuzzy.into_iter().take(LIMIT - exact.len());
		exact.into_iter().chain(it)
	}
}
