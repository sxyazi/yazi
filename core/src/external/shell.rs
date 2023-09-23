use std::{ffi::OsString, process::Stdio};

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
		let expanded_cmd = cmdexpand::expand_cmd(opt.cmd.to_string_lossy().as_ref(), &args)?;
		Ok(
			Command::new("cmd")
				.arg("/C")
				.arg(expanded_cmd)
				.stdin(if opt.piped { Stdio::piped() } else { Stdio::inherit() })
				.stdout(if opt.piped { Stdio::piped() } else { Stdio::inherit() })
				.stderr(if opt.piped { Stdio::piped() } else { Stdio::inherit() })
				.kill_on_drop(true)
				.spawn()?,
		)
	}
}

#[cfg(target_os = "windows")]
mod cmdexpand {
	use anyhow::{anyhow, Result};
	use nom::branch::alt;
	use nom::bytes::complete::{is_not, tag, take_while1};
	use nom::character::complete::{anychar, char, digit1, space0, space1};
	use nom::combinator::recognize;
	use nom::multi::{many0, many1};
	use nom::sequence::{delimited, pair, preceded, tuple};
	use nom::IResult;

	enum CommandPart<'a> {
		Space(&'a str),
		Text(&'a str),
	}

	enum TextPart<'a> {
		NormalText(&'a str),
		PercentNumber(usize),
		PercentStar,
	}

	#[derive(Debug, Copy, Clone)]
	enum Quote {
		DoubleQuote,
		SingleQuote,
		NoQuote,
	}

	pub fn expand_cmd<T>(cmd: &str, args: &[T]) -> Result<String>
	where
		T: AsRef<str>,
	{
		let parts = parse_cmd(cmd)?;
		let mut expanded = String::new();
		for part in parts {
			match part {
				CommandPart::Space(s) => expanded.push_str(s),
				CommandPart::Text(text) => {
					expanded.push_str(&expand_text(text, args)?);
				}
			}
		}
		Ok(expanded)
	}

	fn expand_text<T>(text: &str, args: &[T]) -> Result<String>
	where
		T: AsRef<str>,
	{
		let quote = if text.starts_with("\"") {
			Quote::DoubleQuote
		} else if text.starts_with("'") {
			Quote::SingleQuote
		} else {
			Quote::NoQuote
		};

		let parts = parse_text(text)?;
		let mut expanded = String::new();
		for part in parts {
			match part {
				TextPart::NormalText(s) => expanded.push_str(s),
				TextPart::PercentNumber(i) => {
					if i > 0 {
						let replace_text = args
							.get(i - 1)
							.map(|content| preprocess(content.as_ref(), quote))
							.unwrap_or_default();
						expanded.push_str(&replace_text);
					} else {
						// Does not support %0, replace it with ""
					}
				}
				TextPart::PercentStar => {
					for (i, arg) in args.iter().enumerate() {
						expanded.push_str(&preprocess(arg.as_ref(), quote));
						if i + 1 < args.len() {
							expanded.push_str(" ");
						}
					}
				}
			}
		}
		Ok(expanded)
	}

	fn parse_cmd(cmd: &str) -> Result<Vec<CommandPart>> {
		fn double_quote_text(input: &str) -> IResult<&str, &str> {
			recognize(delimited(char('"'), is_not("\""), char('"')))(input)
		}

		fn single_quote_text(input: &str) -> IResult<&str, &str> {
			recognize(delimited(char('\''), is_not("'"), char('\'')))(input)
		}

		fn no_quote_text(input: &str) -> IResult<&str, &str> {
			take_while1(|c: char| !c.is_whitespace())(input)
		}

		let (_, (leading_space, command_name, args, trailing_space)) = tuple((
			space0,
			alt((double_quote_text, single_quote_text, no_quote_text)),
			many0(pair(space1, alt((double_quote_text, single_quote_text, no_quote_text)))),
			space0,
		))(cmd)
		.map_err(|_| anyhow!("Cannot parse command `{cmd}`"))?;
		let mut parts = Vec::new();
		if !leading_space.is_empty() {
			parts.push(CommandPart::Space(leading_space));
		}
		parts.push(CommandPart::Text(command_name));
		for (space, arg) in args {
			parts.push(CommandPart::Space(space));
			parts.push(CommandPart::Text(arg));
		}
		if !trailing_space.is_empty() {
			parts.push(CommandPart::Space(trailing_space));
		}
		Ok(parts)
	}

	fn parse_text(text: &str) -> Result<Vec<TextPart>> {
		fn escaped_char(input: &str) -> IResult<&str, &str> {
			recognize(pair(char('\\'), anychar))(input)
		}

		fn normal_text(input: &str) -> IResult<&str, TextPart> {
			let (input, output) = recognize(many1(alt((escaped_char, is_not("\\%")))))(input)?;
			Ok((input, TextPart::NormalText(output)))
		}

		fn percent_star(input: &str) -> IResult<&str, TextPart> {
			let (input, _) = tag("%*")(input)?;
			Ok((input, TextPart::PercentStar))
		}

		fn percent_number(input: &str) -> IResult<&str, TextPart> {
			let (input, output) = preceded(char('%'), digit1)(input)?;
			let num: usize = output.parse().unwrap();
			Ok((input, TextPart::PercentNumber(num)))
		}

		let (_, parts) = many0(alt((normal_text, percent_star, percent_number)))(text)
			.map_err(|_| anyhow!("Cannot parse text `{text}`"))?;
		Ok(parts)
	}

	// Preprocess the content inside %x before replacing it in the command text
	// to make sure white space and quote inside %x does not mess up the command.
	fn preprocess(content: &str, quote: Quote) -> String {
		let inner_space = content.chars().any(|c| c.is_whitespace());
		match (quote, inner_space) {
			(Quote::NoQuote, true) => format!("\"{}\"", content.replace("\"", "\\\"")),
			(Quote::NoQuote, false) => content.to_string(),
			(Quote::SingleQuote, _) => content.replace("'", "\\'").to_string(),
			(Quote::DoubleQuote, _) => content.replace("\"", "\\\"").to_string(),
		}
	}
}
