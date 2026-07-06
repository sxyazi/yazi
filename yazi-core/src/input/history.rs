use hashbrown::HashMap;

#[derive(Default)]
pub struct InputHistories {
	inner: HashMap<String, Vec<String>>,
}

impl InputHistories {
	pub fn get(&self, group: &str) -> &[String] {
		self.inner.get(group).map(Vec::as_slice).unwrap_or(&[])
	}

	pub fn remember(&mut self, group: &str, value: &str) -> bool {
		if group.is_empty() || value.is_empty() {
			return false;
		}

		let entries = self.inner.entry_ref(group).or_default();
		if entries.last().is_some_and(|last| last == value) {
			return false;
		}

		entries.retain(|entry| entry != value);
		entries.push(value.to_owned());
		if entries.len() > 20 {
			entries.drain(..entries.len() - 20);
		}

		true
	}
}
