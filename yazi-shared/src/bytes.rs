pub trait BytesExt {
	fn split_by_seq(&self, sep: &[u8]) -> Option<(&[u8], &[u8])>;
}

impl BytesExt for [u8] {
	fn split_by_seq(&self, sep: &[u8]) -> Option<(&[u8], &[u8])> {
		let idx = memchr::memmem::find(self, sep)?;
		let (left, right) = self.split_at(idx);
		Some((left, &right[sep.len()..]))
	}
}

pub fn human_bytes(n: u64) -> String {
	const KB: f64 = 1024.0;
	const MB: f64 = KB * 1024.0;
	const GB: f64 = MB * 1024.0;
	let x = n as f64;
	if x >= GB {
		format!("{:.2} GB", x / GB)
	} else if x >= MB {
		format!("{:.2} MB", x / MB)
	} else if x >= KB {
		format!("{:.2} KB", x / KB)
	} else {
		format!("{n} B")
	}
}
