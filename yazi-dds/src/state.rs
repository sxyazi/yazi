use std::{collections::HashMap, io::{BufRead, BufReader, BufWriter, Write}, mem};

use anyhow::Result;
use parking_lot::Mutex;
use yazi_boot::BOOT;
use yazi_shared::RoCell;

use crate::{body::Body, QUEUE};

pub static STATE: RoCell<Mutex<State>> = RoCell::new();

#[derive(Default)]
pub struct State {
	inner: HashMap<String, String>,
}

impl State {
	pub fn add(&mut self, key: String, value: &str) { self.inner.insert(key, value.to_owned()); }

	pub fn load(&mut self) -> Result<()> {
		let mut buf = BufReader::new(std::fs::File::open(BOOT.state_dir.join("state"))?);
		let mut line = String::new();

		while buf.read_line(&mut line)? > 0 {
			let mut parts = line.splitn(4, ',');
			let Some(kind) = parts.next() else { continue };
			let Some(_) = parts.next() else { continue };
			let Some(severity) = parts.next().and_then(|s| s.parse::<u8>().ok()) else { continue };
			let Some(body) = parts.next() else { continue };

			self.inner.insert(format!("{}_{severity}_{kind}", Body::tab(kind, body)), line.clone());
			QUEUE.send(mem::take(&mut line)).ok();
		}
		Ok(())
	}

	pub fn drain(&mut self) -> Result<()> {
		let mut buf = BufWriter::new(
			std::fs::OpenOptions::new()
				.write(true)
				.create(true)
				.truncate(true)
				.open(BOOT.state_dir.join("state"))?,
		);

		let mut state = mem::take(&mut self.inner).into_iter().collect::<Vec<_>>();
		state.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
		state.into_iter().for_each(|(_, v)| _ = writeln!(buf, "{v}"));
		Ok(())
	}
}
