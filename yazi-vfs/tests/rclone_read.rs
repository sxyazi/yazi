//! Live read/copy-path tests for the rclone provider.
//!
//! These hit a real rclone remote, so they're `#[ignore]` by default and are
//! driven entirely by environment variables — nothing is hard-coded. Set:
//!
//!   YAZI_RCLONE_TEST_SMALL  a small (<1 MiB) text file as `remote:path`
//!   YAZI_RCLONE_TEST_BIG    a >8 MiB file as `remote:path` (spans copy chunks)
//!   YAZI_RCLONE_TEST_DIR    a directory as `remote:path` (has children)
//!
//! Then run:
//!   cargo test -p yazi-vfs --test rclone_read -- --ignored --nocapture

use tokio::io::{AsyncReadExt, AsyncSeekExt};
use yazi_fs::provider::{DirReader, FileHolder};
use yazi_shared::url::UrlBuf;

fn env(key: &str) -> Option<String> { std::env::var(key).ok().filter(|s| !s.is_empty()) }

/// `remote:path/to/obj` → `rclone://remote//path/to/obj`
fn to_url(remote_path: &str) -> String {
	let (remote, path) = remote_path.split_once(':').expect("expected `remote:path`");
	format!("rclone://{remote}//{path}")
}

fn cat(remote_path: &str) -> Vec<u8> {
	std::process::Command::new("rclone").arg("cat").arg(remote_path).output().unwrap().stdout
}

fn init() {
	static ONCE: std::sync::Once = std::sync::Once::new();
	ONCE.call_once(|| {
		yazi_shared::init();
		yazi_fs::init();
		yazi_vfs::init();
	});
}

macro_rules! require {
	($key:literal) => {
		match env($key) {
			Some(v) => v,
			None => {
				eprintln!("skipping: set {} to run this test", $key);
				return;
			}
		}
	};
}

#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn reads_and_seeks() {
	init();
	let small = require!("YAZI_RCLONE_TEST_SMALL");
	let truth = cat(&small);
	assert!(truth.len() > 100, "expected a non-trivial file");

	let url: UrlBuf = to_url(&small).parse().expect("parse rclone url");

	// Full sequential read
	let mut f = yazi_vfs::provider::open(&url).await.expect("open");
	let mut buf = Vec::new();
	f.read_to_end(&mut buf).await.expect("read_to_end");
	assert_eq!(buf, truth, "streamed bytes must match `rclone cat`");

	// Seek to the middle, read the tail, compare
	let mid = (truth.len() / 2) as u64;
	let mut f = yazi_vfs::provider::open(&url).await.expect("reopen");
	f.seek(std::io::SeekFrom::Start(mid)).await.expect("seek");
	let mut tail = Vec::new();
	f.read_to_end(&mut tail).await.expect("read tail");
	assert_eq!(tail, &truth[mid as usize..], "post-seek bytes must match");

	println!("OK reads_and_seeks: {} bytes, seek-tail {} bytes", buf.len(), tail.len());
}

/// The remote root lists buckets; a directory path lists its children.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn lists_root_and_dir() {
	init();
	let dir = require!("YAZI_RCLONE_TEST_DIR");

	async fn names(url: &str) -> Vec<String> {
		let url: UrlBuf = url.parse().unwrap();
		let mut rd = yazi_vfs::provider::read_dir(&url).await.expect("read_dir");
		let mut out = Vec::new();
		while let Some(e) = rd.next().await.expect("next") {
			out.push(e.name().into_string_lossy());
		}
		out
	}

	let remote = dir.split_once(':').unwrap().0;
	let buckets = names(&format!("rclone://{remote}//")).await;
	assert!(!buckets.is_empty(), "remote root should list at least one bucket");

	let children = names(&to_url(&dir)).await;
	assert!(!children.is_empty(), "directory should list its children");

	println!("OK lists_root_and_dir: {} buckets, {} children", buckets.len(), children.len());
}

/// Opening a nonexistent object errors. Note: object stores cannot distinguish
/// a missing key from an empty directory prefix, so `rclone lsjson --stat`
/// reports the path as a directory — hence `InvalidInput` ("Is a directory")
/// rather than `NotFound`. Either way, the open fails cleanly instead of
/// streaming garbage.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn missing_object_errors() {
	init();
	let dir = require!("YAZI_RCLONE_TEST_DIR");
	let remote = dir.split_once(':').unwrap().0;

	let url: UrlBuf =
		to_url(&format!("{remote}:this/object/does/not/exist_xyz.bin")).parse().unwrap();
	match yazi_vfs::provider::open(&url).await {
		Ok(_) => panic!("expected an error opening a nonexistent object"),
		Err(e) => println!("OK missing_object_errors: {} ({:?})", e, e.kind()),
	}
}

/// Simple (non-progress) download path: `copy_impl` → open(rclone) +
/// create(local).
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn copy_out_simple() {
	init();
	let small = require!("YAZI_RCLONE_TEST_SMALL");
	let truth = cat(&small);
	let from: UrlBuf = to_url(&small).parse().unwrap();

	let dir = std::env::temp_dir().join("yazi_rclone_test_simple");
	std::fs::create_dir_all(&dir).unwrap();
	let dest = dir.join("out.bin");
	let to: UrlBuf = dest.to_str().unwrap().parse().unwrap();

	let n = yazi_vfs::provider::copy(&from, &to, Default::default()).await.expect("copy");
	let got = std::fs::read(&dest).unwrap();
	assert_eq!(got, truth, "downloaded bytes must match");
	assert_eq!(n as usize, truth.len(), "reported byte count must match");
	std::fs::remove_dir_all(&dir).ok();
	println!("OK copy_out_simple: {n} bytes");
}

/// Progressive chunked download path: `ProgressiveCopier` (parallel 8 MiB
/// chunks
/// + per-chunk seek). A >8 MiB file spans multiple chunks.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn copy_out_progressive() {
	init();
	let big = require!("YAZI_RCLONE_TEST_BIG");
	let truth = cat(&big);
	assert!(truth.len() > 8 * 1024 * 1024, "need a >8 MiB file to span chunks");
	let from: UrlBuf = to_url(&big).parse().unwrap();

	let dir = std::env::temp_dir().join("yazi_rclone_test_prog");
	std::fs::create_dir_all(&dir).unwrap();
	let dest = dir.join("out.bin");
	let to: UrlBuf = dest.to_str().unwrap().parse().unwrap();

	let mut rx =
		yazi_vfs::provider::copy_with_progress(&from, &to, yazi_fs::provider::Attrs::default())
			.await
			.expect("copy_with_progress");

	let mut total = 0u64;
	while let Some(msg) = rx.recv().await {
		total += msg.expect("no copy error");
	}

	let got = std::fs::read(&dest).unwrap();
	assert_eq!(got.len(), truth.len(), "downloaded size must match");
	assert_eq!(got, truth, "downloaded bytes must match byte-for-byte");
	std::fs::remove_dir_all(&dir).ok();
	println!("OK copy_out_progressive: {} bytes (progress sum {total})", got.len());
}
