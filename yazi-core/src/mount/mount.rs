use std::{io::BufRead, path::PathBuf, sync::Arc, time::Duration};

use parking_lot::Mutex;
use yazi_adapter::Dimension;
use yazi_scheduler::{Ongoing, TaskSummary};

use super::{MOUNT_BORDER, MOUNT_PADDING, MOUNT_PERCENT};

#[derive(Debug)]
pub struct MountPoint {
	pub dev: String,
	pub path: PathBuf,
	pub fs: String,
	pub opts: String,
}

#[derive(Default)]
pub struct Mount {
	pub visible: bool,
	pub cursor: usize,

	pub points: Vec<MountPoint>,
}

impl Mount {
	pub fn update(&mut self) {
		let points =
			std::io::BufReader::new(std::fs::File::open(PathBuf::from("/proc/mounts")).unwrap())
				.lines()
				.map_while(Result::ok)
				.filter_map(|l| {
					let mut parts = l.trim_end_matches(" 0 0").split(' ');
					Some(MountPoint {
						dev: parts.next()?.into(),
						path: parts.next()?.into(),
						fs: parts.next()?.into(),
						opts: parts.next()?.into(),
					})
				})
				.filter(|p| !p.path.starts_with("/sys"))
				.filter(|p| !p.path.starts_with("/tmp"))
				.filter(|p| !p.path.starts_with("/run"))
				.filter(|p| !p.path.starts_with("/dev"))
				.filter(|p| !p.path.starts_with("/proc"));
		self.points = points.collect();
	}

	#[inline]
	pub fn limit() -> usize {
		(Dimension::available().rows * MOUNT_PERCENT / 100).saturating_sub(MOUNT_BORDER + MOUNT_PADDING)
			as usize
	}
}
