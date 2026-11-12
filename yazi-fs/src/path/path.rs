use yazi_shared::{strand::StrandCow, url::Url};

pub fn skip_url<'a>(url: impl Into<Url<'a>>, n: usize) -> StrandCow<'a> {
	let mut it = url.into().components();
	for _ in 0..n {
		if it.next().is_none() {
			return StrandCow::default();
		}
	}
	it.strand()
}
