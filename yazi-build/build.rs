use std::{env, error::Error, io::{BufRead, BufReader, Read, Write}, process::{Command, Stdio}, thread};

use yazi_term::tty::TTY;

fn main() -> Result<(), Box<dyn Error>> {
	yazi_term::init();

	let manifest = env::var_os("CARGO_MANIFEST_DIR").unwrap().to_string_lossy().replace(r"\", "/");
	let crates = if manifest.contains("/git/checkouts/yazi-") {
		&["--git", "https://github.com/sxyazi/yazi.git", "yazi-fm", "yazi-cli"]
	} else if manifest.contains("/registry/src/index.crates.io-") {
		&["yazi-fm", "yazi-cli"][..]
	} else {
		return Ok(());
	};

	let target = env::var("TARGET").unwrap();
	let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
	unsafe {
		env::set_var("VERGEN_GIT_SHA", "Crates.io");
		env::set_var("YAZI_CRATE_BUILD", "1");

		env::set_var("JEMALLOC_SYS_WITH_LG_PAGE", "16");
		env::set_var("JEMALLOC_SYS_WITH_MALLOC_CONF", "narenas:1");

		env::set_var(
			"MACOSX_DEPLOYMENT_TARGET",
			if target == "aarch64-apple-darwin" { "11.0" } else { "10.12" },
		);
		if target == "aarch64-apple-darwin" {
			env::set_var("RUSTFLAGS", "-Ctarget-cpu=apple-m1");
		}
	};

	let profile = if target_os == "windows" { &["--profile", "release-windows"][..] } else { &[] };
	let mut child = Command::new(env::var_os("CARGO").unwrap())
		.args(["install", "--force", "--locked"])
		.args(profile)
		.args(crates)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;

	let out = flash(child.stdout.take().unwrap());
	let err = flash(child.stderr.take().unwrap());

	child.wait()?;
	out.join().ok();
	err.join().ok();

	Ok(())
}

fn flash<R: Read + Send + 'static>(src: R) -> thread::JoinHandle<()> {
	thread::spawn(move || {
		let reader = BufReader::new(src);
		for part in reader.split(b'\n') {
			match part {
				Ok(mut bytes) => {
					bytes.push(b'\n');
					let mut out = TTY.lockout();
					out.write_all(&bytes).ok();
					out.flush().ok();
				}
				Err(_) => break,
			}
		}
	})
}
