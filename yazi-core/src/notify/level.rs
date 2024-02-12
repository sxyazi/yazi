use std::str::FromStr;

pub enum Level {
	Info,
	Warn,
	Error,
}

impl FromStr for Level {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"info" => Self::Info,
			"warn" => Self::Warn,
			"error" => Self::Error,
			_ => return Err(()),
		})
	}
}
