use std::time::Duration;

use anyhow::{bail, Result};
use tokio::{io::{stdin, AsyncReadExt, BufReader}, time::timeout};
use tracing::error;

use super::Term;

impl Term {
	pub async fn read_until_da1() -> Result<String> {
		let read = async {
			let mut stdin = BufReader::new(stdin());
			let mut buf = String::with_capacity(200);
			loop {
				let mut c = [0; 1];
				if stdin.read(&mut c).await? == 0 {
					bail!("unexpected EOF");
				}
				buf.push(c[0] as char);
				if c[0] == b'c' && buf.contains("\x1b[?") {
					break;
				}
			}
			Ok(buf)
		};

		let timeout = timeout(Duration::from_secs(10), read).await;
		if let Err(ref e) = timeout {
			error!("read_until_da1: {e:?}");
		}

		timeout?
	}
}
