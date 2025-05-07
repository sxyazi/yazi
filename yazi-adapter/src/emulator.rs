use std::{io::BufWriter, time::Duration};

use anyhow::Result;
use crossterm::{cursor::{RestorePosition, SavePosition}, execute, style::Print, terminal::{disable_raw_mode, enable_raw_mode}};
use scopeguard::defer;
use tokio::time::sleep;
use tracing::{debug, error, warn};
use yazi_shared::Either;
use yazi_term::tty::{Handle, TTY};

use crate::{Adapter, Brand, Dimension, Mux, TMUX, Unknown};

#[derive(Clone, Copy, Debug)]
pub struct Emulator {
	pub kind:      Either<Brand, Unknown>,
	pub light:     bool,
	pub csi_16t:   (u16, u16),
	pub force_16t: bool,
}

impl Default for Emulator {
	fn default() -> Self { Self::unknown() }
}

impl Emulator {
	pub fn detect() -> Result<Self> {
		defer! { disable_raw_mode().ok(); }
		enable_raw_mode()?;

		let resort = Brand::from_env();
		let kgp_seq = if resort.is_none() {
			Mux::csi("\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\")
		} else {
			"".into()
		};

		execute!(
			TTY.writer(),
			SavePosition,
			Print(kgp_seq),             // Detect KGP
			Print(Mux::csi("\x1b[>q")), // Request terminal version
			Print("\x1b[16t"),          // Request cell size
			Print("\x1b]11;?\x07"),     // Request background color
			Print(Mux::csi("\x1b[0c")), // Request device attributes
			RestorePosition
		)?;

		let resp = Self::read_until_da1();
		Mux::tmux_drain()?;

		let kind = if let Some(b) = Brand::from_csi(&resp).or(resort) {
			Either::Left(b)
		} else {
			Either::Right(Unknown {
				kgp:   resp.contains("\x1b_Gi=31;OK"),
				sixel: ["?4;", "?4c", ";4;", ";4c"].iter().any(|s| resp.contains(s)),
			})
		};

		let csi_16t = Self::csi_16t(&resp).unwrap_or_default();
		Ok(Self {
			kind,
			light: Self::light_bg(&resp).unwrap_or_default(),
			csi_16t,
			force_16t: Self::force_16t(csi_16t),
		})
	}

	pub const fn unknown() -> Self {
		Self {
			kind:      Either::Right(Unknown::default()),
			light:     false,
			csi_16t:   (0, 0),
			force_16t: false,
		}
	}

	pub fn adapters(self) -> &'static [Adapter] {
		match self.kind {
			Either::Left(brand) => brand.adapters(),
			Either::Right(unknown) => unknown.adapters(),
		}
	}

	pub fn move_lock<F, T>((x, y): (u16, u16), cb: F) -> Result<T>
	where
		F: FnOnce(&mut BufWriter<Handle>) -> Result<T>,
	{
		use std::{io::Write, thread, time::Duration};

		use crossterm::{cursor::{Hide, MoveTo, Show}, queue};

		let mut w = TTY.lockout();

		// I really don't want to add this,
		// But tmux and ConPTY sometimes cause the cursor position to get out of sync.
		if TMUX.get() || cfg!(windows) {
			execute!(w, SavePosition, MoveTo(x, y), Show)?;
			execute!(w, MoveTo(x, y), Show)?;
			execute!(w, MoveTo(x, y), Show)?;
			thread::sleep(Duration::from_millis(1));
		} else {
			queue!(w, SavePosition, MoveTo(x, y))?;
		}

		let result = cb(&mut w);
		if TMUX.get() || cfg!(windows) {
			queue!(w, Hide, RestorePosition)?;
		} else {
			queue!(w, RestorePosition)?;
		}

		w.flush()?;
		result
	}

	pub fn read_until_da1() -> String {
		let now = std::time::Instant::now();
		let h = tokio::spawn(Self::error_to_user());

		let (buf, result) = TTY.read_until(Duration::from_millis(500), |b, buf| {
			b == b'c'
				&& buf.contains(&0x1b)
				&& buf.rsplitn(2, |&b| b == 0x1b).next().is_some_and(|s| s.starts_with(b"[?"))
		});

		h.abort();
		match result {
			Ok(()) => debug!("Terminal responded to DA1 in {:?}: {buf:?}", now.elapsed()),
			Err(e) => {
				error!("Terminal failed to respond to DA1 in {:?}: {buf:?}, error: {e:?}", now.elapsed())
			}
		}

		String::from_utf8_lossy(&buf).into_owned()
	}

	pub fn read_until_dsr() -> String {
		let now = std::time::Instant::now();
		let (buf, result) = TTY.read_until(Duration::from_millis(100), |b, buf| {
			b == b'n' && (buf.ends_with(b"\x1b[0n") || buf.ends_with(b"\x1b[3n"))
		});

		match result {
			Ok(()) => debug!("Terminal responded to DSR in {:?}: {buf:?}", now.elapsed()),
			Err(e) => {
				error!("Terminal failed to respond to DSR in {:?}: {buf:?}, error: {e:?}", now.elapsed())
			}
		}
		String::from_utf8_lossy(&buf).into_owned()
	}

	async fn error_to_user() {
		use crossterm::style::{Attribute, Color, Print, ResetColor, SetAttributes, SetForegroundColor};

		sleep(Duration::from_millis(400)).await;
		_ = crossterm::execute!(
			std::io::stderr(),
			SetForegroundColor(Color::Red),
			SetAttributes(Attribute::Bold.into()),
			Print("\r\nTerminal response timeout: "),
			ResetColor,
			SetAttributes(Attribute::Reset.into()),
			//
			Print("The request sent by Yazi didn't receive a correct response.\r\n"),
			Print(
				"Please check your terminal environment as per: https://yazi-rs.github.io/docs/faq#trt\r\n"
			),
		);
	}

	fn csi_16t(resp: &str) -> Option<(u16, u16)> {
		let b = resp.split_once("\x1b[6;")?.1.as_bytes();

		let h: Vec<_> = b.iter().copied().take_while(|&c| c.is_ascii_digit()).collect();
		b.get(h.len()).filter(|&&c| c == b';')?;

		let w: Vec<_> = b[h.len() + 1..].iter().copied().take_while(|&c| c.is_ascii_digit()).collect();
		b.get(h.len() + 1 + w.len()).filter(|&&c| c == b't')?;

		let (w, h) = unsafe { (String::from_utf8_unchecked(w), String::from_utf8_unchecked(h)) };
		Some((w.parse().ok()?, h.parse().ok()?))
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

	fn force_16t((w, h): (u16, u16)) -> bool {
		if w == 0 || h == 0 {
			return false;
		}

		Dimension::available()
			.ratio()
			.is_none_or(|(rw, rh)| rw.floor() as u16 != w || rh.floor() as u16 != h)
	}
}
