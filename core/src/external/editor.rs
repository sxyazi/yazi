use std::{env, ffi::{OsStr, OsString}, path::Path, str::FromStr};

use anyhow::{Context, Result};
use once_cell::sync::OnceCell;
use tokio::process::Command;

#[derive(Debug)]
struct Editor {
	name:    String,
	options: Vec<String>,
}

static EDITOR: OnceCell<Editor> = OnceCell::new();

fn try_init() -> Result<&'static Editor> {
	EDITOR.get_or_try_init(|| {
		env::var_os("EDITOR")
			.context("environment variable `EDITOR` is undefined")?
			.to_string_lossy()
			.parse()
	})
}

pub async fn edit(file: impl AsRef<Path>) -> Result<()> {
	let editor = try_init()?;
	// TODO: 编辑并返回是否成功
	// let output = Command::new(&editor.name)
	// 	.args(&editor.options)
	// 	.arg(file.as_ref())
	// 	.kill_on_drop(true)
	// 	.output()
	// 	.await?;
	todo!()
}

impl FromStr for Editor {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		let mut args = s.split(' ');
		Ok(Self {
			name:    args.next().context("environment variable `EDITOR` is empty")?.to_owned(),
			options: args.map(str::to_owned).collect(),
		})
	}
}
