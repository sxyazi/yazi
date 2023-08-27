use super::File;

pub struct NonHiddenFiles<'a> {
	items: &'a Vec<File>,

	cur: usize,
	max: usize,
}

impl<'a> NonHiddenFiles<'a> {
	pub fn new(items: &'a Vec<File>, max: usize) -> Self { Self { items, cur: 0, max } }
}

impl<'a> Iterator for NonHiddenFiles<'a> {
	type Item = &'a File;

	fn next(&mut self) -> Option<Self::Item> {
		while self.cur < self.items.len() {
			let item = &self.items[self.cur];
			self.cur += 1;
			if !item.is_hidden {
				return Some(&item);
			}
		}
		None
	}

	fn size_hint(&self) -> (usize, Option<usize>) { (self.max, Some(self.max)) }
}
