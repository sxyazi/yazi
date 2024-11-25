use std::{io::{LineWriter, stderr}, time::Duration};

use anyhow::{Result, bail};
use crossterm::{cursor::{RestorePosition, SavePosition}, execute, style::Print, terminal::{disable_raw_mode, enable_raw_mode}};
use scopeguard::defer;
use tokio::{io::{AsyncReadExt, BufReader}, time::timeout};
use tracing::{debug, error, warn};
use yazi_shared::Either;

use crate::{Adapter, Brand, Mux, TMUX, Unknown};

#[derive(Clone, Copy, Debug)]
pub struct Emulator {
	pub kind:  Either<Brand, Unknown>,
	pub light: bool,
}

impl Default for Emulator {
	fn default() -> Self { Self { kind: Either::Right(Unknown::default()), light: false } }
}

impl Emulator {
	pub fn detect() -> Self {
		if let Some(brand) = Brand::from_env() {
			Self { kind: Either::Left(brand), light: Self::detect_base().unwrap_or_default().light }
		} else {
			Self::detect_full().unwrap_or_default()
		}
	}

	pub fn detect_full() -> Result<Self> {
		defer! { disable_raw_mode().ok(); }
		enable_raw_mode()?;

		execute!(
			LineWriter::new(stderr()),
			SavePosition,
			Print(Mux::csi("\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\")), // Detect KGP
			Print(Mux::csi("\x1b[>q")),                                    // Request terminal version
			Print(Mux::csi("\x1b[c")),                                     // Request device attributes
			RestorePosition
		)?;

		let resp = futures::executor::block_on(Self::read_until_da1());
		let kind = if let Some(brand) = Brand::from_csi(&resp) {
			Either::Left(brand)
		} else {
			Either::Right(Unknown {
				kgp:   resp.contains("\x1b_Gi=31;OK"),
				sixel: ["?4;", "?4c", ";4;", ";4c"].iter().any(|s| resp.contains(s)),
			})
		};

		Ok(Self { kind, light: Self::light_bg(&resp).unwrap_or_default() })
	}

	pub fn adapters(self) -> &'static [Adapter] {
		match self.kind {
			Either::Left(brand) => brand.adapters(),
			Either::Right(unknown) => unknown.adapters(),
		}
	}

	pub fn move_lock<F, T>((x, y): (u16, u16), cb: F) -> Result<T>
	where
		F: FnOnce(&mut std::io::BufWriter<std::io::StderrLock>) -> Result<T>,
	{
		use std::{io::Write, thread, time::Duration};

		use crossterm::{cursor::{Hide, MoveTo, RestorePosition, SavePosition, Show}, queue};

		let mut buf = std::io::BufWriter::new(stderr().lock());

		// I really don't want to add this,
		// But tmux and ConPTY sometimes cause the cursor position to get out of sync.
		if *TMUX || cfg!(windows) {
			execute!(buf, SavePosition, MoveTo(x, y), Show)?;
			execute!(buf, MoveTo(x, y), Show)?;
			execute!(buf, MoveTo(x, y), Show)?;
			thread::sleep(Duration::from_millis(1));
		} else {
			queue!(buf, SavePosition, MoveTo(x, y))?;
		}

		let result = cb(&mut buf);
		if *TMUX || cfg!(windows) {
			queue!(buf, Hide, RestorePosition)?;
		} else {
			queue!(buf, RestorePosition)?;
		}

		buf.flush()?;
		result
	}

	pub async fn read_until_da1() -> String {
		let mut buf: Vec<u8> = Vec::with_capacity(200);
		let read = async {
			let mut stdin = BufReader::new(tokio::io::stdin());
			loop {
				let mut c = [0; 1];
				if stdin.read(&mut c).await? == 0 {
					bail!("unexpected EOF");
				}
				buf.push(c[0]);
				if c[0] != b'c' || !buf.contains(&0x1b) {
					continue;
				}
				if buf.rsplitn(2, |&b| b == 0x1b).next().is_some_and(|s| s.starts_with(b"[?")) {
					break;
				}
			}
			Ok(())
		};

		match timeout(Duration::from_secs(10), read).await {
			Err(e) => error!("read_until_da1 timed out: {buf:?}, error: {e:?}"),
			Ok(Err(e)) => error!("read_until_da1 failed: {buf:?}, error: {e:?}"),
			Ok(Ok(())) => {}
		}
		String::from_utf8_lossy(&buf).into_owned()
	}

	fn detect_base() -> Result<Self> {
		defer! { disable_raw_mode().ok(); }
		enable_raw_mode()?;

		execute!(
			LineWriter::new(stderr()),
			Print(Mux::csi("\x1b]11;?\x07")), // Request background color
			Print(Mux::csi("\x1b[c")),        // Request device attributes
		)?;

		let resp = futures::executor::block_on(Self::read_until_da1());
		Ok(Self { light: Self::light_bg(&resp).unwrap_or_default(), ..Default::default() })
	}

	fn light_bg(resp: &str) -> Result<bool> {
		match resp.split_once("]11;rgb:") {
			Some((_, s)) if s.len() >= 14 => {
				let r = u8::from_str_radix(&s[0..2], 16)? as f32;
				let g = u8::from_str_radix(&s[5..7], 16)? as f32;
				let b = u8::from_str_radix(&s[10..12], 16)? as f32;
				let luma = r * 0.2627 / 256.0 + g * 0.6780 / 256.0 + b * 0.0593 / 256.0;
				debug!("Detected background color: {} (luma = {luma:.2})", &s[..14]);
				Ok(luma > 0.6)
			}
			_ => {
				warn!("Failed to detect background color: {resp:?}");
				Ok(false)
			}
		}
	}
}
