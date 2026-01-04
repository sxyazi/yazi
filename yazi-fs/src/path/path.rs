use yazi_shared::{strand::StrandCow, url::{UrlBuf, UrlLike}};

pub fn skip_url(url: &UrlBuf, n: usize) -> StrandCow<'_> {
	let mut it = url.components();
	for _ in 0..n {
		if it.next().is_none() {
			return StrandCow::default();
		}
	}
	it.strand()
}
