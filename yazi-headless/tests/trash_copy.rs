#![cfg(target_os = "linux")]

use std::{fs, os::unix::fs::PermissionsExt, process::Command, time::{Duration, SystemTime}};

use yazi_shared::url::{UrlBuf, UrlLike};
use yazi_vfs::engine;

#[test]
fn copies_trashed_contents() {
	if std::env::var_os("YAZI_TEST_TRASH_CHILD").is_none() {
		let sandbox = tempfile::tempdir().unwrap();
		let status = Command::new(std::env::current_exe().unwrap())
			.args(["--exact", "copies_trashed_contents", "--nocapture"])
			.env("YAZI_TEST_TRASH_CHILD", "1")
			.env("HOME", sandbox.path())
			.env("XDG_DATA_HOME", sandbox.path().join("data"))
			.env("XDG_CACHE_HOME", sandbox.path().join("cache"))
			.env("XDG_STATE_HOME", sandbox.path().join("state"))
			.env("XDG_RUNTIME_DIR", sandbox.path().join("run"))
			.env("YAZI_CONFIG_HOME", sandbox.path().join("config"))
			.status()
			.unwrap();
		assert!(status.success(), "trash copy regression failed in isolated process");
		return;
	}

	tokio::runtime::Runtime::new().unwrap().block_on(async {
		let _services = yazi_headless::init().await.unwrap();
		let root = std::path::PathBuf::from(std::env::var_os("HOME").unwrap());
		let trash = root.join("data/Trash");
		fs::create_dir_all(trash.join("info")).unwrap();
		fs::create_dir_all(trash.join("files")).unwrap();

		for (name, size) in [("empty", 0), ("small", 123), ("large", 9 * 1024 * 1024)] {
			let content: Vec<_> = (0..size).map(|i| (i % 251) as u8).collect();
			let backing = trash.join("files").join(name);
			fs::write(&backing, &content).unwrap();
			fs::set_permissions(&backing, fs::Permissions::from_mode(0o640)).unwrap();
			let modified = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
			fs::File::open(&backing)
				.unwrap()
				.set_times(fs::FileTimes::new().set_modified(modified))
				.unwrap();
			let info = trash.join("info").join(format!("{name}.trashinfo"));
			fs::write(&info, format!("[Trash Info]\nPath={}/{}\n", root.display(), name)).unwrap();

			let source: UrlBuf =
				format!("trash://{}/@//{name}", info.to_str().unwrap().replace('/', "%2F"))
					.parse()
					.unwrap();
			let target: UrlBuf = root.join(format!("restored-{name}")).into();
			let stat = engine::metadata(&source).await.unwrap();
			assert_eq!(stat.len, size as u64);
			let copied = engine::copy(&source, &target, stat).await.unwrap().total().await.unwrap();
			assert_eq!(copied, size as u64);
			assert_eq!(fs::read(target.as_local().unwrap()).unwrap(), content);
			let restored = fs::metadata(target.as_local().unwrap()).unwrap();
			assert_eq!(restored.permissions().mode() & 0o777, 0o640);
			assert_eq!(restored.modified().unwrap(), modified);
			assert_eq!(fs::read(&backing).unwrap(), content, "copy must retain the trash entry");

			let moved: UrlBuf = root.join(format!("moved-{name}")).into();
			engine::copy(&source, &moved, stat).await.unwrap().total().await.unwrap();
			engine::remove_file(&source).await.unwrap();
			assert_eq!(fs::read(moved.as_local().unwrap()).unwrap(), content);
			assert!(!backing.exists());
			assert!(!info.exists());

			let missing: UrlBuf = root.join(format!("missing-{name}")).into();
			assert!(engine::copy(&source, &missing, stat).await.unwrap().total().await.is_err());
			assert!(!missing.as_local().unwrap().exists());
		}
	});
}
