use std::{env, ffi::OsString, path::PathBuf};

use anyhow::{Context, Result, anyhow, bail, ensure};

pub(super) enum Args {
	Build(String),
	Dest(String),
	Install(PathBuf),
	Help,
}

impl Args {
	pub(super) fn parse() -> Result<Self> {
		let mut args = env::args_os().skip(1);
		let task = args.next().unwrap_or_else(|| "--help".into());

		let mut target = String::new();
		let mut bin_dir = PathBuf::new();
		while let Some(arg) = args.next() {
			match arg.to_str() {
				Some("--target") => target = Self::target(&mut args)?,
				Some("--bin-dir") => bin_dir = Self::value(&mut args, "--bin-dir")?.into(),
				Some("--help") => return Ok(Self::Help),
				_ => bail!("unknown option: {}", arg.display()),
			}
		}

		match task.to_str() {
			Some("build") => {
				ensure!(bin_dir.as_os_str().is_empty(), "--bin-dir is only valid for the install task");
				Ok(Self::Build(target))
			}
			Some("dist") => {
				ensure!(!target.is_empty(), "the dist task requires --target");
				ensure!(bin_dir.as_os_str().is_empty(), "--bin-dir is only valid for the install task");
				Ok(Self::Dest(target))
			}
			Some("install") => {
				ensure!(target.is_empty(), "--target is not valid for the install task");
				Ok(Self::Install(bin_dir))
			}
			Some("--help") => Ok(Self::Help),
			_ => bail!("unknown task: {}", task.display()),
		}
	}

	fn value(args: &mut impl Iterator<Item = OsString>, option: &str) -> Result<OsString> {
		args.next().with_context(|| format!("missing value for {option}"))
	}

	fn target(args: &mut impl Iterator<Item = OsString>) -> Result<String> {
		Self::value(args, "--target")?
			.into_string()
			.map_err(|_| anyhow!("--target must be valid UTF-8"))
	}
}
