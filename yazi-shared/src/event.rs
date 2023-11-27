use std::{any::Any, cell::RefCell, collections::BTreeMap, fmt::{self, Display}};

#[derive(Debug, Default)]
pub struct Exec {
	pub cmd:   String,
	pub args:  Vec<String>,
	pub named: BTreeMap<String, String>,
	pub data:  RefCell<Option<Box<dyn Any + Send>>>,
}

impl Exec {
	#[inline]
	pub fn call(cwd: &str, args: Vec<String>) -> Self {
		Exec { cmd: cwd.to_owned(), args, ..Default::default() }
	}

	#[inline]
	pub fn call_named(cwd: &str, named: BTreeMap<String, String>) -> Self {
		Exec { cmd: cwd.to_owned(), named, ..Default::default() }
	}

	#[inline]
	pub fn vec(self) -> Vec<Self> { vec![self] }

	#[inline]
	pub fn with(mut self, name: impl ToString, value: impl ToString) -> Self {
		self.named.insert(name.to_string(), value.to_string());
		self
	}

	#[inline]
	pub fn with_bool(mut self, name: impl ToString, state: bool) -> Self {
		if state {
			self.named.insert(name.to_string(), Default::default());
		}
		self
	}

	#[inline]
	pub fn with_data(mut self, data: impl Any + Send) -> Self {
		self.data = RefCell::new(Some(Box::new(data)));
		self
	}
}

impl Display for Exec {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.cmd)?;
		if !self.args.is_empty() {
			write!(f, " {}", self.args.join(" "))?;
		}
		for (k, v) in &self.named {
			write!(f, " --{k}")?;
			if !v.is_empty() {
				write!(f, "={v}")?;
			}
		}
		Ok(())
	}
}
