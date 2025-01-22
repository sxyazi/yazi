use std::{borrow::Cow, collections::{HashMap, HashSet}, ffi::{OsStr, OsString}, os::{fd::AsFd, unix::{ffi::{OsStrExt, OsStringExt}, fs::MetadataExt}}, time::Duration};

use anyhow::Result;
use tokio::{io::{Interest, unix::AsyncFd}, time::sleep};
use tracing::error;
use yazi_shared::{replace_cow, replace_vec_cow};

use super::{Locked, Partition, Partitions};

impl Partitions {
	pub fn monitor<F>(me: Locked, cb: F)
	where
		F: Fn() + Copy + Send + 'static,
	{
		async fn wait_mounts(me: Locked, cb: impl Fn()) -> Result<()> {
			let f = std::fs::File::open("/proc/mounts")?;
			let fd = AsyncFd::with_interest(f.as_fd(), Interest::READABLE)?;
			loop {
				let mut guard = fd.readable().await?;
				guard.clear_ready();
				Partitions::update(me.clone()).await;
				cb();
			}
		}

		async fn wait_partitions(me: Locked, cb: impl Fn()) -> Result<()> {
			loop {
				let partitions = Partitions::partitions()?;
				if me.read().linux_cache == partitions {
					sleep(Duration::from_secs(3)).await;
					continue;
				}

				me.write().linux_cache = partitions;
				Partitions::update(me.clone()).await;

				cb();
				sleep(Duration::from_secs(3)).await;
			}
		}

		let me_ = me.clone();
		tokio::spawn(async move {
			loop {
				if let Err(e) = wait_mounts(me_.clone(), cb).await {
					error!("Error encountered while monitoring `/proc/mounts`: {e:?}");
				}
				sleep(Duration::from_secs(5)).await;
			}
		});

		tokio::spawn(async move {
			loop {
				if let Err(e) = wait_partitions(me.clone(), cb).await {
					error!("Error encountered while monitoring `/proc/partitions`: {e:?}");
				}
				sleep(Duration::from_secs(5)).await;
			}
		});
	}

	async fn update(me: Locked) {
		_ = tokio::task::spawn_blocking(move || {
			let mut guard = me.write();
			match Self::all(&guard) {
				Ok(new) => guard.inner = new,
				Err(e) => error!("Error encountered while updating mount points: {e:?}"),
			};
		})
		.await;
	}

	fn all(&self) -> Result<Vec<Partition>> {
		let mut mounts = Self::mounts()?;
		{
			let set = &self.linux_cache;
			let mut set: HashSet<&OsStr> = set.iter().map(AsRef::as_ref).collect();
			mounts.iter().filter_map(|p| p.dev_name()).for_each(|s| _ = set.remove(s));
			mounts.extend(set.into_iter().map(Partition::new));
		};

		let labels = Self::labels()?;
		for mount in &mut mounts {
			if !mount.src.as_bytes().starts_with(b"/dev/") {
				continue;
			}
			if let Ok(meta) = std::fs::metadata(&mount.src) {
				mount.rdev = Some(meta.rdev() as _);
				mount.label = labels.get(&(meta.dev(), meta.ino())).cloned();
			}
		}
		Ok(mounts)
	}

	fn mounts() -> Result<Vec<Partition>> {
		let mut vec = vec![];
		let s = std::fs::read_to_string("/proc/mounts")?;
		for line in s.lines() {
			let mut it = line.split_whitespace();
			let Some(src) = it.next() else { continue };
			let Some(dist) = it.next() else { continue };
			let Some(fstype) = it.next() else { continue };
			vec.push(Partition {
				src: Self::unmangle_octal(src).into_owned().into(),
				dist: Some(Self::unmangle_octal(dist).into_owned().into()),
				fstype: Some(Self::unmangle_octal(fstype).into_owned().into()),
				..Default::default()
			});
		}
		Ok(vec)
	}

	fn partitions() -> Result<HashSet<String>> {
		let mut set = HashSet::new();
		let s = std::fs::read_to_string("/proc/partitions")?;
		for line in s.lines().skip(2) {
			let mut it = line.split_whitespace();
			let Some(Ok(_major)) = it.next().map(|s| s.parse::<u16>()) else { continue };
			let Some(Ok(_minor)) = it.next().map(|s| s.parse::<u16>()) else { continue };
			let Some(Ok(_blocks)) = it.next().map(|s| s.parse::<u32>()) else { continue };
			if let Some(name) = it.next() {
				set.insert(Self::unmangle_octal(name).into_owned());
			}
		}
		Ok(set)
	}

	fn labels() -> Result<HashMap<(u64, u64), OsString>> {
		let mut map = HashMap::new();
		for entry in std::fs::read_dir("/dev/disk/by-label")?.flatten() {
			let meta = std::fs::metadata(entry.path())?;
			let name = entry.file_name();
			map.insert(
				(meta.dev(), meta.ino()),
				match replace_vec_cow(name.as_bytes(), br"\x20", b" ") {
					Cow::Borrowed(_) => name,
					Cow::Owned(v) => OsString::from_vec(v),
				},
			);
		}
		Ok(map)
	}

	// Unmangle '\t', '\n', ' ', '#', and r'\'
	// https://elixir.bootlin.com/linux/v6.13-rc3/source/fs/proc_namespace.c#L89
	fn unmangle_octal(s: &str) -> Cow<str> {
		let mut s = Cow::Borrowed(s);
		for (a, b) in
			[(r"\011", "\t"), (r"\012", "\n"), (r"\040", " "), (r"\043", "#"), (r"\134", r"\")]
		{
			s = match replace_cow(&s, a, b) {
				Cow::Borrowed(_) => s,
				Cow::Owned(new) => Cow::Owned(new),
			};
		}
		s
	}
}
