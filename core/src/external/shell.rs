use std::{ffi::OsString, process::Stdio};

use anyhow::Result;
use tokio::process::{Child, Command};

pub struct ShellOpt {
	pub cmd: OsString,
	pub args: Vec<OsString>,
	pub piped: bool,
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
	#[cfg(unix)]
	return Ok(
		Command::new("sh")
			.arg("-c")
			.stdin(opt.stdio())
			.stdout(opt.stdio())
			.stderr(opt.stdio())
			.arg(opt.cmd)
			.arg("") // $0 is the command name
			.args(opt.args)
			.kill_on_drop(!opt.orphan)
			.spawn()?,
	);

	#[cfg(target_os = "windows")]
	{
		let args: Vec<String> = opt.args.iter().map(|s| s.to_string_lossy().to_string()).collect();
		let expanded_args = cmdparse::parse_cmd_to_args(opt.cmd.to_string_lossy().as_ref(), &args);
		Ok(
			Command::new("cmd")
				.arg("/C")
				.args(&expanded_args)
				.stdin(if opt.piped { Stdio::piped() } else { Stdio::inherit() })
				.stdout(if opt.piped { Stdio::piped() } else { Stdio::inherit() })
				.stderr(if opt.piped { Stdio::piped() } else { Stdio::inherit() })
				.kill_on_drop(true)
				.spawn()?,
		)
	}
}

#[cfg(target_os = "windows")]
mod cmdparse {
	pub fn parse_cmd_to_args<T>(cmd: &str, args: &[T]) -> Vec<String>
	where
		T: AsRef<str>,
	{
		let mut iter = cmd.chars().peekable();
		let mut expanded_args = Vec::new();

		while let Some(c) = iter.peek() {
			if c.is_whitespace() {
				while iter.peek().is_some_and(|_c| _c.is_whitespace()) {
					iter.next();
				}
			} else if *c == '\'' {
				iter.next();
				let mut text = String::new();
				loop {
					if iter.peek().is_none() {
						break;
					}
					if iter.peek().is_some_and(|_c| *_c == '\'') {
						iter.next();
						break;
					}
					get_next_char(&mut iter, &mut text, args);
				}
				expanded_args.push(text);
			} else if *c == '"' {
				iter.next();
				let mut text = String::new();
				loop {
					if iter.peek().is_none() {
						break;
					}
					if iter.peek().is_some_and(|_c| *_c == '"') {
						iter.next();
						break;
					}
					get_next_char(&mut iter, &mut text, args);
				}
				expanded_args.push(text);
			} else {
				if *c == '%' {
					let mut tmp_iter = iter.clone();
					tmp_iter.next();
					if tmp_iter.peek().is_some_and(|_c| *_c == '*') {
						iter.next();
						iter.next();
						for arg in args {
							expanded_args.push(arg.as_ref().to_string())
						}
						continue;
					}
				}

				let mut text = String::new();
				loop {
					if iter.peek().is_none() || iter.peek().is_some_and(|_c| _c.is_whitespace()) {
						break;
					}
					get_next_char(&mut iter, &mut text, args);
				}
				expanded_args.push(text);
			}
		}

		expanded_args
	}

	fn get_next_char<T>(
		iter: &mut std::iter::Peekable<std::str::Chars<'_>>,
		text: &mut String,
		args: &[T],
	) where
		T: AsRef<str>,
	{
		let ch = iter.next().unwrap();
		if ch == '\\' {
			match iter.next() {
				Some('n') => text.push('\n'),
				Some('r') => text.push('\r'),
				Some('t') => text.push('\t'),
				Some(x) => text.push(x),
				None => (),
			}
		} else if ch == '%' {
			if iter.peek().is_some_and(|_c| *_c == '*') {
				iter.next();
				text.push_str(&args.iter().map(|value| value.as_ref()).collect::<Vec<&str>>().join(" "));
			} else {
				let mut num = String::new();
				while iter.peek().is_some_and(|_c| _c.is_numeric()) {
					num.push(iter.next().unwrap());
				}
				if num.is_empty() {
					text.push('%');
				} else {
					let i: usize = num.parse().unwrap();
					if i > 0 {
						text.push_str(args.get(i - 1).map(|value| value.as_ref()).unwrap_or_default());
					}
				}
			}
		} else {
			text.push(ch);
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn test_no_quote() {
			let args = parse_cmd_to_args("echo abc xyz %1 %2", &["111", "222"]);
			assert_eq!(args, vec!["echo", "abc", "xyz", "111", "222"]);

			let args = parse_cmd_to_args("  echo   abc   xyz %1   %2  ", &["111", "222"]);
			assert_eq!(args, vec!["echo", "abc", "xyz", "111", "222"]);
		}

		#[test]
		fn test_single_quote() {
			let args = parse_cmd_to_args("echo 'abc xyz' '%1' %2", &["111", "222"]);
			assert_eq!(args, vec!["echo", "abc xyz", "111", "222"]);

			let args = parse_cmd_to_args("echo 'abc \"\"xyz' '%1' %2", &["111", "222"]);
			assert_eq!(args, vec!["echo", "abc \"\"xyz", "111", "222"]);
		}

		#[test]
		fn test_double_quote() {
			let args = parse_cmd_to_args("echo \"abc ' 'xyz\" \"%1\" %2 %3", &["111", "222"]);
			assert_eq!(args, vec!["echo", "abc ' 'xyz", "111", "222", ""]);
		}

		#[test]
		fn test_escaped() {
			let args = parse_cmd_to_args("echo \"a\tbc ' 'x\nyz\" \"\\%1\" %2 %3", &["111", "22  2"]);
			assert_eq!(args, vec!["echo", "a\tbc ' 'x\nyz", "%1", "22  2", ""]);
		}

		#[test]
		fn test_percent_star() {
			let args = parse_cmd_to_args("echo %* xyz", &["111", "222"]);
			assert_eq!(args, vec!["echo", "111", "222", "xyz"]);

			let args = parse_cmd_to_args("echo '%*' xyz", &["111", "222"]);
			assert_eq!(args, vec!["echo", "111 222", "xyz"]);

			let args = parse_cmd_to_args("echo -C%* xyz", &["111", "222"]);
			assert_eq!(args, vec!["echo", "-C111 222", "xyz"]);
		}

		#[test]
		fn test_env_var() {
			let args = parse_cmd_to_args(" %EDITOR% %* xyz", &["111", "222"]);
			assert_eq!(args, vec!["%EDITOR%", "111", "222", "xyz"]);
		}
	}
}
