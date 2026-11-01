use std::{borrow::Cow, ffi::{OsStr, OsString}, fs, os::unix::{ffi::{OsStrExt, OsStringExt}, fs::MetadataExt}};

use anyhow::{Context, Result};
use hashbrown::{HashMap, HashSet};
use rustix::{io, net::{self, AddressFamily, SocketFlags, SocketType, netlink::{KOBJECT_UEVENT, SocketAddrNetlink}}};
use tokio::{io::{Interest, unix::AsyncFd}, join, task};
use yazi_macro::{error, log_if_err};
use yazi_shared::{natsort, replace_cow, replace_vec_cow};

use super::{Locked, Partition, Partitions};

impl Partitions {
	pub fn monitor(me: &'static Locked, cb: fn()) {
		tokio::spawn(async move {
			Self::update(me, cb).await;

			join!(
				async move { log_if_err!("Monitoring /proc/mounts", Self::wait_mounts(me, cb).await) },
				async move { log_if_err!("Monitoring partition events", Self::wait_partitions(me, cb).await) }
			);
		});
	}

	async fn wait_mounts(me: &'static Locked, cb: fn()) -> Result<()> {
		let f = fs::File::open("/proc/mounts")?;
		let fd = unsafe { AsyncFd::register_with_interest(f, Interest::READABLE) }?;
		loop {
			let mut guard = fd.readable().await?;
			guard.clear_ready();
			Self::update(me, cb).await;
		}
	}

	async fn wait_partitions(me: &'static Locked, cb: fn()) -> Result<()> {
		let socket = net::socket_with(
			AddressFamily::NETLINK,
			SocketType::DGRAM,
			SocketFlags::CLOEXEC | SocketFlags::NONBLOCK,
			Some(KOBJECT_UEVENT),
		)?;

		net::bind(&socket, &SocketAddrNetlink::new(0, 1))?;
		let socket = unsafe { AsyncFd::register_with_interest(socket, Interest::READABLE) }?;

		let mut buf = [0; 16 * 1024];
		loop {
			let mut ready = socket.readable().await?;
			let mut refresh = false;
			for _ in 0..64 {
				let result = ready.try_io(|fd| io::read(fd.get_ref(), &mut buf).map_err(Into::into));
				refresh |= match result {
					Err(_) => break,
					Ok(Ok(len)) => {
						len == buf.len() || buf[..len].split(|&b| b == 0).any(|s| s == b"SUBSYSTEM=block")
					}
					Ok(Err(e)) if e.raw_os_error() == Some(libc::ENOBUFS) => true,
					Ok(Err(e)) => return Err(e.into()),
				};
			}

			drop(ready);
			if refresh {
				Self::update(me, cb).await;
			}

			task::yield_now().await;
		}
	}

	fn partitions() -> Result<HashSet<OsString>> {
		let mut set = HashSet::new();
		let s = fs::read_to_string("/proc/partitions")?;
		for line in s.lines().skip(2) {
			let mut it = line.split_whitespace();
			let Some(Ok(_major)) = it.next().map(|s| s.parse::<u16>()) else { continue };
			let Some(Ok(_minor)) = it.next().map(|s| s.parse::<u16>()) else { continue };
			let Some(Ok(_blocks)) = it.next().map(|s| s.parse::<u64>()) else { continue };
			if let Some(name) = it.next() {
				set.insert(Self::unmangle_octal(name).into_owned());
			}
		}
		Ok(set)
	}

	async fn update(me: &'static Locked, cb: fn()) {
		match task::spawn_blocking(Self::all).await {
			Ok(Ok(new)) => me.write().set(new),
			r => error!("Error encountered while updating partitions: {r:?}"),
		}
		cb();
	}

	pub(super) fn all() -> Result<Vec<Partition>> {
		let mut mounts = Self::mounts().context("Parsing /proc/mounts")?;
		let mut partitions = Self::partitions().context("Parsing /proc/partitions")?;

		mounts.iter().filter_map(|p| p.dev_name(true)).for_each(|s| _ = partitions.remove(s));
		mounts.extend(partitions.into_iter().map(Partition::new));
		mounts.sort_unstable_by(|a, b| natsort(a.src.as_bytes(), b.src.as_bytes(), false));

		let labels = Self::labels();
		let removable = Self::removable(&mounts);

		for mount in &mut mounts {
			if !mount.src.as_bytes().starts_with(b"/dev/") {
				continue;
			}
			if let Ok(meta) = fs::metadata(&mount.src) {
				mount.rdev = Some(meta.rdev() as _);
				mount.label = labels.get(&(meta.dev(), meta.ino())).cloned();
				// TODO: mount.external
				mount.removable = mount.dev_name(false).and_then(|s| removable.get(s).copied()).flatten();
			}
		}

		Ok(mounts)
	}

	fn mounts() -> Result<Vec<Partition>> {
		let mut vec = vec![];
		let s = fs::read_to_string("/proc/mounts")?;
		for line in s.lines() {
			let mut it = line.split_whitespace();
			let Some(src) = it.next() else { continue };
			let Some(dist) = it.next() else { continue };
			let Some(fstype) = it.next() else { continue };
			vec.push(Partition {
				src: Self::unmangle_octal(src).into_owned(),
				dist: Some(Self::unmangle_octal(dist).into_owned().into()),
				fstype: Some(Self::unmangle_octal(fstype).into_owned()),
				..Default::default()
			});
		}
		Ok(vec)
	}

	fn labels() -> HashMap<(u64, u64), OsString> {
		let mut map = HashMap::new();
		let Ok(it) = fs::read_dir("/dev/disk/by-label") else {
			error!("Cannot read /dev/disk/by-label");
			return map;
		};

		for dent in it.flatten() {
			let Ok(meta) = fs::metadata(dent.path()) else { continue };
			let name = dent.file_name();
			map.insert(
				(meta.dev(), meta.ino()),
				match replace_vec_cow(name.as_bytes(), br"\x20", b" ") {
					Cow::Borrowed(_) => name,
					Cow::Owned(v) => OsString::from_vec(v),
				},
			);
		}
		map
	}

	fn removable(mounts: &[Partition]) -> HashMap<OsString, Option<bool>> {
		let mut removable: HashMap<OsString, Option<bool>> =
			mounts.iter().filter_map(|p| p.dev_name(false)).map(|s| (s.to_owned(), None)).collect();

		for (s, b) in &mut removable {
			*b = match fs::read(format!("/sys/block/{}/removable", s.display()))
				.unwrap_or_default()
				.trim_ascii()
			{
				b"0" => Some(false),
				b"1" => Some(true),
				_ => None,
			};
		}

		removable
	}

	// Unmangle '\t', '\n', ' ', '#', and r'\'
	// https://elixir.bootlin.com/linux/v6.13-rc3/source/fs/proc_namespace.c#L89
	fn unmangle_octal(s: &str) -> Cow<'_, OsStr> {
		let mut s = Cow::Borrowed(s);
		for (a, b) in
			[(r"\011", "\t"), (r"\012", "\n"), (r"\040", " "), (r"\043", "#"), (r"\134", r"\")]
		{
			s = replace_cow(s, a, b);
		}
		match s {
			Cow::Borrowed(s) => Cow::Borrowed(OsStr::new(s)),
			Cow::Owned(s) => Cow::Owned(s.into()),
		}
	}
}
