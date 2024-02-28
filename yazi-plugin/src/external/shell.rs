use std::{env, ffi::OsString, process::Stdio};

use anyhow::Result;
use tokio::process::{Child, Command};

pub struct ShellOpt {
	pub cmd:    OsString,
	pub args:   Vec<OsString>,
	pub piped:  bool,
	pub orphan: bool,
}

impl ShellOpt {
	pub fn with_piped(mut self) -> Self {
		self.piped = true;
		self
	}

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
	let level = env::var("YAZI_LEVEL").ok().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);

	#[cfg(unix)]
	return Ok(unsafe {
		Command::new("sh")
			.env("YAZI_LEVEL", (level + 1).to_string())
			.arg("-c")
			.stdin(opt.stdio())
			.stdout(opt.stdio())
			.stderr(opt.stdio())
			.arg(opt.cmd)
			.args(opt.args)
			.kill_on_drop(!opt.orphan)
			.pre_exec(move || {
				if opt.orphan && libc::setpgid(0i32, 0i32) < 0 {
					libc::perror(std::ptr::null());
				}
				Ok(())
			})
			.spawn()?
	});

	#[cfg(windows)]
	{
		let args: Vec<String> = opt.args.iter().map(|s| s.to_string_lossy().into_owned()).collect();
		let args_: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();
		let expanded = parser::parse(opt.cmd.to_string_lossy().as_ref(), &args_);
		Ok(
			Command::new("cmd")
				.env("YAZI_LEVEL", (level + 1).to_string())
				.arg("/C")
				.args(&expanded)
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
	use std::{iter::Peekable, str::Chars};

	pub(super) fn parse(cmd: &str, args: &[&str]) -> Vec<String> {
		let mut it = cmd.chars().peekable();
		let mut expanded = vec![];

		while let Some(c) = it.next() {
			if c.is_whitespace() {
				continue;
			}
			let mut s = String::new();

			if c == '\'' {
				while let Some(c) = it.next() {
					if c == '\'' {
						break;
					}
					next_string(&mut it, args, &mut s, c);
				}
				expanded.push(s);
			} else if c == '"' {
				while let Some(c) = it.next() {
					if c == '"' {
						break;
					}
					next_string(&mut it, args, &mut s, c);
				}
				expanded.push(s);
			} else if c == '%' && it.peek().is_some_and(|&c| c == '*') {
				it.next();
				for arg in args {
					expanded.push(arg.to_string());
				}
			} else {
				next_string(&mut it, args, &mut s, c);

				while let Some(c) = it.next() {
					if c.is_whitespace() {
						break;
					}
					next_string(&mut it, args, &mut s, c);
				}
				expanded.push(s);
			}
		}

		expanded
	}

	fn next_string(it: &mut Peekable<Chars<'_>>, args: &[&str], s: &mut String, c: char) {
		if c == '\\' {
			match it.next() {
				Some('\\') => s.push('\\'), // \\  ==>  \
				Some('\'') => s.push('\''), // \'  ==>  '
				Some('"') => s.push('"'),   // \"  ==>  "
				Some('%') => s.push('%'),   // \%  ==>  %
				Some('n') => s.push('\n'),  // \n  ==>  '\n'
				Some('t') => s.push('\t'),  // \t  ==>  '\t'
				Some('r') => s.push('\r'),  // \r  ==>  '\r'
				Some(c) => {
					s.push('\\');
					s.push(c);
				}
				None => s.push('\\'),
			}
		} else if c == '%' {
			match it.peek() {
				Some('*') => {
					s.push_str(&args.join(" "));
					it.next();
				}
				Some(n) if n.is_ascii_digit() => {
					let mut pos = n.to_string();

					it.next();
					while let Some(&n) = it.peek() {
						if n.is_ascii_digit() {
							pos.push(it.next().unwrap());
						} else {
							break;
						}
					}

					let pos = pos.parse::<usize>().unwrap();
					if pos > 0 {
						s.push_str(args.get(pos - 1).unwrap_or(&""));
					}
				}
				_ => s.push('%'),
			}
		} else {
			s.push(c);
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn test_no_quote() {
			let args = parse("echo abc xyz %0 %2", &["111", "222"]);
			assert_eq!(args, ["echo", "abc", "xyz", "", "222"]);

			let args = parse("  echo   abc   xyz %1   %2  ", &["111", "222"]);
			assert_eq!(args, ["echo", "abc", "xyz", "111", "222"]);
		}

		#[test]
		fn test_single_quote() {
			let args = parse("echo 'abc xyz' '%1' %2", &["111", "222"]);
			assert_eq!(args, ["echo", "abc xyz", "111", "222"]);

			let args = parse(r#"echo 'abc ""xyz' '%1' %2"#, &["111", "222"]);
			assert_eq!(args, ["echo", r#"abc ""xyz"#, "111", "222"]);
		}

		#[test]
		fn test_double_quote() {
			let args = parse("echo \"abc ' 'xyz\" \"%1\" %2 %3", &["111", "222"]);
			assert_eq!(args, ["echo", "abc ' 'xyz", "111", "222", ""]);
		}

		#[test]
		fn test_escaped() {
			let args = parse("echo \"a\tbc ' 'x\nyz\" \"\\%1\" %2 %3", &["111", "22  2"]);
			assert_eq!(args, ["echo", "a\tbc ' 'x\nyz", "%1", "22  2", ""]);
		}

		#[test]
		fn test_percent_star() {
			let args = parse("echo %* xyz", &["111", "222"]);
			assert_eq!(args, ["echo", "111", "222", "xyz"]);

			let args = parse("echo '%*' xyz", &["111", "222"]);
			assert_eq!(args, ["echo", "111 222", "xyz"]);

			let args = parse("echo -C%* xyz", &["111", "222"]);
			assert_eq!(args, ["echo", "-C111 222", "xyz"]);
		}

		#[test]
		fn test_env_var() {
			let args = parse(" %EDITOR% %* xyz", &["111", "222"]);
			assert_eq!(args, ["%EDITOR%", "111", "222", "xyz"]);
		}
	}
}
