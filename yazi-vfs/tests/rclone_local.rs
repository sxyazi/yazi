//! Hermetic tests for the rclone provider.
//!
//! Unix-only: they drive an rclone `local` remote with absolute filesystem
//! paths, which the (Unix-style) rclone scheme can't represent on Windows.
//! Cloud remotes work fine from any OS — this is a limitation of the test's
//! local backend, not the provider.
//!
//! These need no cloud account and no credentials: they point an rclone `local`
//! remote at a generated temp directory, so `rclone lsjson` / `rclone cat` run
//! entirely against the filesystem. The only requirement is the `rclone` binary
//! on PATH; if it's missing the tests skip themselves (so CI without rclone
//! stays green, and CI that installs rclone gets real coverage).

#![cfg(unix)]

use std::{path::{Path, PathBuf}, process::Command, sync::OnceLock};

use tokio::io::{AsyncReadExt, AsyncSeekExt};
use yazi_fs::provider::{DirReader, FileHolder};
use yazi_shared::url::UrlBuf;

fn have_rclone() -> bool {
	Command::new("rclone").arg("version").output().map(|o| o.status.success()).unwrap_or(false)
}

/// Byte at position `i` of the big fixture — varies per offset so that a wrong
/// `--offset` or a botched seek produces detectably wrong bytes.
fn big_byte(i: usize) -> u8 { (i % 251) as u8 }

const SMALL: &[u8] = b"hello rclone vfs\nsecond line\nthird line\n";
const BIG_LEN: usize = 9 * 1024 * 1024 + 777; // > 8 MiB, non-aligned

/// One-time fixture + config setup for the whole test binary. Returns the data
/// directory, or `None` if rclone is unavailable. Must run before any provider
/// call, because it sets `YAZI_CONFIG_HOME` before yazi caches it.
fn setup() -> Option<&'static Path> {
	static SETUP: OnceLock<Option<PathBuf>> = OnceLock::new();
	SETUP
		.get_or_init(|| {
			if !have_rclone() {
				return None;
			}

			let root = std::env::temp_dir().join(format!("yazi_rclone_local_{}", std::process::id()));
			let data = root.join("data");
			std::fs::create_dir_all(data.join("sub")).unwrap();
			std::fs::write(data.join("small.txt"), SMALL).unwrap();
			std::fs::write(data.join("sub").join("nested.txt"), b"nested\n").unwrap();
			let big: Vec<u8> = (0..BIG_LEN).map(big_byte).collect();
			std::fs::write(data.join("big.bin"), &big).unwrap();

			let conf = root.join("rclone.conf");
			std::fs::write(&conf, "[local]\ntype = local\n").unwrap();

			let cfg = root.join("config");
			std::fs::create_dir_all(&cfg).unwrap();
			std::fs::write(
				cfg.join("vfs.toml"),
				format!(
					"[services.local]\ntype = \"rclone\"\nremote = \"local\"\nconfig_file = {:?}\n",
					conf.to_str().unwrap()
				),
			)
			.unwrap();

			// SAFETY: set before any threads touch the environment or yazi init.
			unsafe { std::env::set_var("YAZI_CONFIG_HOME", &cfg) };
			yazi_shared::init();
			yazi_fs::init();
			yazi_vfs::init();

			Some(data)
		})
		.as_deref()
}

macro_rules! data_or_skip {
	() => {
		match setup() {
			Some(d) => d,
			None => {
				eprintln!("skipping: `rclone` not found on PATH");
				return;
			}
		}
	};
}

fn url(abs: &Path) -> UrlBuf { format!("rclone://local/{}", abs.display()).parse().unwrap() }

#[tokio::test(flavor = "multi_thread")]
async fn reads_and_seeks() {
	let data = data_or_skip!();
	let u = url(&data.join("small.txt"));

	let mut f = yazi_vfs::provider::open(&u).await.expect("open");
	let mut buf = Vec::new();
	f.read_to_end(&mut buf).await.expect("read");
	assert_eq!(buf, SMALL);

	let mid = SMALL.len() / 2;
	let mut f = yazi_vfs::provider::open(&u).await.expect("reopen");
	f.seek(std::io::SeekFrom::Start(mid as u64)).await.expect("seek");
	let mut tail = Vec::new();
	f.read_to_end(&mut tail).await.expect("read tail");
	assert_eq!(tail, &SMALL[mid..]);
	println!("OK reads_and_seeks");
}

#[tokio::test(flavor = "multi_thread")]
async fn lists_dir() {
	let data = data_or_skip!();
	let mut rd = yazi_vfs::provider::read_dir(&url(data)).await.expect("read_dir");
	let mut names = Vec::new();
	while let Some(e) = rd.next().await.expect("next") {
		names.push(e.name().into_string_lossy());
	}
	names.sort();
	assert_eq!(names, vec!["big.bin", "small.txt", "sub"]);
	println!("OK lists_dir: {names:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn missing_object_errors() {
	let data = data_or_skip!();
	let u = url(&data.join("does_not_exist.bin"));
	match yazi_vfs::provider::open(&u).await {
		Ok(_) => panic!("expected an error"),
		Err(e) => println!("OK missing_object_errors: {} ({:?})", e, e.kind()),
	}
}

/// Progressive chunked copy (parallel 8 MiB chunks + per-chunk seek) of the
/// >8 MiB fixture — the byte-varying content makes any offset/seek slip
/// visible.
#[tokio::test(flavor = "multi_thread")]
async fn copy_out_progressive() {
	let data = data_or_skip!();
	let from = url(&data.join("big.bin"));

	// Write outside the fixture `data/` dir so it doesn't perturb `lists_dir`.
	let out = data.parent().unwrap().join("copied.bin");
	let to: UrlBuf = out.to_str().unwrap().parse().unwrap();

	let mut rx =
		yazi_vfs::provider::copy_with_progress(&from, &to, yazi_fs::provider::Attrs::default())
			.await
			.expect("copy_with_progress");
	let mut total = 0u64;
	while let Some(msg) = rx.recv().await {
		total += msg.expect("no copy error");
	}

	let got = std::fs::read(&out).unwrap();
	assert_eq!(got.len(), BIG_LEN, "size mismatch");
	assert!(got.iter().enumerate().all(|(i, &b)| b == big_byte(i)), "byte mismatch after copy");
	println!("OK copy_out_progressive: {BIG_LEN} bytes (progress sum {total})");
}
