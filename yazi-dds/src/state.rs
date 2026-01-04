use std::{mem, ops::Deref, sync::atomic::{AtomicU64, Ordering}};

use anyhow::Result;
use hashbrown::HashMap;
use parking_lot::RwLock;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use yazi_boot::BOOT;
use yazi_fs::provider::{FileBuilder, Provider, local::{Gate, Local}};
use yazi_shared::{RoCell, timestamp_us};

use crate::CLIENTS;

pub static STATE: RoCell<State> = RoCell::new();

#[derive(Default)]
pub struct State {
	inner: RwLock<Option<HashMap<String, String>>>,
	last:  AtomicU64,
}

impl Deref for State {
	type Target = RwLock<Option<HashMap<String, String>>>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl State {
	pub fn set(&self, kind: &str, sender: u64, body: &str) -> bool {
		let Some(inner) = &mut *self.inner.write() else { return false };

		if body == "null" {
			return inner
				.remove(kind)
				.map(|_| self.last.store(timestamp_us(), Ordering::Relaxed))
				.is_some();
		}

		let value = format!("{kind},0,{sender},{body}\n");
		if inner.get(kind).is_some_and(|s| *s == value) {
			return false;
		}

		inner.insert(kind.to_owned(), value);
		self.last.store(timestamp_us(), Ordering::Relaxed);
		true
	}

	pub async fn load_or_create(&self) {
		if self.load().await.is_err() {
			self.inner.write().replace(Default::default());
			self.last.store(timestamp_us(), Ordering::Relaxed);
		}
	}

	pub async fn drain(&self) -> Result<()> {
		let Some(inner) = self.inner.write().take() else { return Ok(()) };
		if self.skip().await.unwrap_or(false) {
			return Ok(());
		}

		Local::regular(&BOOT.state_dir).create_dir_all().await?;
		let mut buf = BufWriter::new(
			Gate::default()
				.write(true)
				.create(true)
				.truncate(true)
				.open(BOOT.state_dir.join(".dds"))
				.await?,
		);

		let mut state = inner.into_iter().collect::<Vec<_>>();
		state.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
		for (_, v) in state {
			buf.write_all(v.as_bytes()).await?;
		}

		buf.flush().await?;
		Ok(())
	}

	async fn load(&self) -> Result<()> {
		let mut file = BufReader::new(Local::regular(&BOOT.state_dir.join(".dds")).open().await?);
		let mut buf = String::new();

		let mut inner = HashMap::new();
		while file.read_line(&mut buf).await? > 0 {
			let line = mem::take(&mut buf);
			let mut parts = line.splitn(4, ',');

			let Some(kind) = parts.next() else { continue };
			let Some(_) = parts.next() else { continue };
			inner.insert(kind.to_owned(), line);
		}

		let clients = CLIENTS.read();
		for payload in inner.values() {
			clients.values().for_each(|c| _ = c.tx.send(payload.clone()));
		}

		self.inner.write().replace(inner);
		self.last.store(timestamp_us(), Ordering::Relaxed);
		Ok(())
	}

	async fn skip(&self) -> Result<bool> {
		let cha = Local::regular(&BOOT.state_dir.join(".dds")).symlink_metadata().await?;
		let modified = cha.mtime_dur()?.as_micros();
		Ok(modified >= self.last.load(Ordering::Relaxed) as u128)
	}
}
