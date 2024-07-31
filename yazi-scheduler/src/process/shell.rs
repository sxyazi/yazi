use std::{ffi::OsString, process::Stdio};

use anyhow::Result;
use tokio::process::{Child, Command};

#[derive(Default)]
pub struct ShellOpt {
	pub cmd:    OsString,
	pub args:   Vec<OsString>,
	pub piped:  bool,
	pub orphan: bool,
}

impl ShellOpt {
	#[inline]
	fn stdio(&self) -> Stdio {
		if self.orphan {
			Stdio::null()
		} else if self.piped {
			Stdio::piped()
		} else {
			Stdio::inherit()
		}
	}
}

pub fn shell(opt: ShellOpt) -> Result<Child> {
	#[cfg(unix)]
	return Ok(unsafe {
		Command::new("sh")
			.arg("-c")
			.stdin(opt.stdio())
			.stdout(opt.stdio())
			.stderr(opt.stdio())
			.arg(opt.cmd)
			.args(opt.args)
			.kill_on_drop(!opt.orphan)
			.pre_exec(move || {
				if opt.orphan && libc::setpgid(0, 0) < 0 {
					return Err(std::io::Error::last_os_error());
				}
				Ok(())
			})
			.spawn()?
	});

	#[cfg(windows)]
	{
		Ok(
			Command::new("cmd.exe")
				.raw_arg("/C")
				.raw_arg(parser::parse(&opt.cmd, &opt.args))
				.stdin(opt.stdio())
				.stdout(opt.stdio())
				.stderr(opt.stdio())
				.kill_on_drop(!opt.orphan)
				.spawn()?,
		)
	}
}

#[cfg(windows)]
mod parser {
	use std::{ffi::{OsStr, OsString}, iter::Peekable, os::windows::ffi::{EncodeWide, OsStrExt, OsStringExt}};

	macro_rules! w {
		($c:literal) => {
			$c as u16
		};
	}

	pub(super) fn parse(cmd: &OsStr, args: &[OsString]) -> OsString {
		if cmd.len() < 2 {
			return cmd.to_owned();
		}

		let mut buf = Vec::with_capacity(cmd.len());
		let mut it = cmd.encode_wide().peekable();

		while let Some(c) = it.next() {
			if c == w!('%') {
				let n = visit_percent(it.clone(), &mut buf, &args, false);
				if n == 0 {
					buf.push(c);
				} else {
					it.by_ref().take(n).for_each(drop);
				}
			} else if c == w!('"') && it.peek().is_some_and(|&c| c == w!('%')) {
				it.next();

				let n = visit_percent(it.clone(), &mut buf, &args, true);
				if n == 0 {
					buf.push(c);
					buf.push(w!('%'));
				} else {
					it.by_ref().take(n).for_each(drop);
				}
			} else {
				buf.push(c);
			}
		}

		OsString::from_wide(&buf)
	}

	fn visit_percent(
		mut it: Peekable<EncodeWide>,
		buf: &mut Vec<u16>,
		args: &[OsString],
		quote: bool,
	) -> usize {
		let Some(c) = it.next().and_then(|c| char::from_u32(c as _)) else {
			return 0;
		};

		let mut pos = None;
		if c.is_ascii_digit() {
			let mut p = c.to_string();
			while let Some(n) = it.peek().and_then(|&c| char::from_u32(c as _)) {
				if n.is_ascii_digit() {
					it.next();
					p.push(n);
				} else {
					break;
				}
			}
			pos = Some(p);
		}

		if quote && !it.next().is_some_and(|e| e == w!('"')) {
			return 0;
		}

		if let Some(p) = pos {
			if let Some(arg) = args.get(p.parse::<usize>().unwrap()) {
				if quote {
					buf.extend(yazi_shared::shell::escape_os_str(arg).encode_wide());
				} else {
					buf.extend(arg.encode_wide());
				}
			}
			return p.len() + quote as usize;
		}

		if c != '*' && c != '@' {
			return 0;
		}

		let mut s = OsString::new();
		for (i, arg) in args.iter().skip(1).enumerate() {
			if i > 0 {
				s.push(" ");
			}
			if c == '*' {
				s.push(yazi_shared::shell::escape_os_str(arg));
			} else {
				s.push(arg);
			}
		}
		if quote {
			buf.extend(yazi_shared::shell::escape_os_str(&s).encode_wide());
		} else {
			buf.extend(s.encode_wide());
		}

		1 + quote as usize
	}

	#[cfg(test)]
	mod tests {
		use std::ffi::OsString;

		fn parse(cmd: &str, args: &[&str]) -> String {
			let cmd = OsString::from(cmd);
			let args: Vec<_> = args.iter().map(|&s| OsString::from(s)).collect();
			super::parse(&cmd, &args).to_str().unwrap().to_owned()
		}

		#[test]
		fn test_no_quote() {
			let s = parse("echo abc xyz %0 %2", &["000", "111", "222"]);
			assert_eq!(s, "echo abc xyz 000 222");

			let s = parse("  echo   abc   xyz %1   %2  ", &["", "111", "222"]);
			assert_eq!(s, "  echo   abc   xyz 111   222  ");
		}

		#[test]
		fn test_single_quote() {
			let s = parse("echo 'abc xyz' '%1' %2", &["000", "111", "222"]);
			assert_eq!(s, "echo 'abc xyz' '111' 222");

			let s = parse(r#"echo 'abc ""xyz' '%1' %2"#, &["", "111", "222"]);
			assert_eq!(s, r#"echo 'abc ""xyz' '111' 222"#);
		}

		#[test]
		fn test_double_quote() {
			let s = parse(r#"echo "abc ' 'xyz" "%1" %2 %3"#, &["", "111", "222"]);
			assert_eq!(s, r#"echo "abc ' 'xyz" 111 222 "#);
		}

		#[test]
		fn test_escaped() {
			let s = parse(r#"echo "a	bc ' 'x\nyz" "\%1" "\"%2"" %3"#, &["", "111", "22  2"]);
			assert_eq!(s, r#"echo "a	bc ' 'x\nyz" "\111" "\"22  2"" "#);
		}

		#[test]
		fn test_percent_star() {
			let s = parse("echo %* xyz", &[]);
			assert_eq!(s, "echo  xyz");

			let s = parse("echo %* xyz", &["000", "111", "222"]);
			assert_eq!(s, "echo 111 222 xyz");

			let s = parse("echo '%*' xyz", &["000", "111", "22 2"]);
			assert_eq!(s, r#"echo '111 "22 2"' xyz"#);

			let s = parse("echo -C%* xyz", &[]);
			assert_eq!(s, "echo -C xyz");

			let s = parse("echo -C%* xyz", &["000", " 111", "222"]);
			assert_eq!(s, r#"echo -C" 111" 222 xyz"#);
		}

		#[test]
		fn test_env_var() {
			let s = parse(r#"%EDITOR% %@ "%@" %* "%*" xyz"#, &["000", "1 11", "222"]);
			assert_eq!(s, r#"%EDITOR% 1 11 222 "1 11 222" "1 11" 222 "\"1 11\" 222" xyz"#);
		}
	}
}
