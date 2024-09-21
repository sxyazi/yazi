use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Deserializer, de};

#[derive(Debug, PartialEq, Eq)]
pub enum ConditionOp {
	Or,
	And,
	Not,

	// Only used in `build()`
	Space,
	LeftParen,
	RightParen,
	Unknown,

	// Only used in `eval()`
	Term(String),
}

impl ConditionOp {
	pub fn new(c: char) -> Self {
		match c {
			'|' => Self::Or,
			'&' => Self::And,
			'!' => Self::Not,

			'(' => Self::LeftParen,
			')' => Self::RightParen,
			_ if c.is_ascii_whitespace() => Self::Space,
			_ => Self::Unknown,
		}
	}

	#[inline]
	pub fn prec(&self) -> u8 {
		match self {
			ConditionOp::Or => 1,
			ConditionOp::And => 2,
			ConditionOp::Not => 3,
			_ => 0,
		}
	}
}

#[derive(Debug)]
pub struct Condition {
	ops: Vec<ConditionOp>,
}

impl FromStr for Condition {
	type Err = anyhow::Error;

	fn from_str(expr: &str) -> Result<Self, Self::Err> {
		let cond = Self::build(expr);
		if cond.eval(|_| true).is_none() {
			bail!("Invalid condition: {expr}");
		}

		Ok(cond)
	}
}

impl<'de> Deserialize<'de> for Condition {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let s = String::deserialize(deserializer)?;
		FromStr::from_str(&s).map_err(de::Error::custom)
	}
}

impl Condition {
	fn build(expr: &str) -> Self {
		let mut stack: Vec<ConditionOp> = vec![];
		let mut output: Vec<ConditionOp> = vec![];

		let mut chars = expr.chars().peekable();
		while let Some(token) = chars.next() {
			let op = ConditionOp::new(token);
			match op {
				ConditionOp::Or | ConditionOp::And | ConditionOp::Not => {
					while matches!(stack.last(), Some(last) if last.prec() >= op.prec()) {
						output.push(stack.pop().unwrap());
					}
					stack.push(op);
				}
				ConditionOp::Space => continue,
				ConditionOp::LeftParen => stack.push(op),
				ConditionOp::RightParen => {
					while matches!(stack.last(), Some(last) if last != &ConditionOp::LeftParen) {
						output.push(stack.pop().unwrap());
					}
					stack.pop();
				}
				ConditionOp::Unknown => {
					let mut s = String::from(token);
					while matches!(chars.peek(), Some(&c) if ConditionOp::new(c) == op) {
						s.push(chars.next().unwrap());
					}
					output.push(ConditionOp::Term(s));
				}
				ConditionOp::Term(_) => unreachable!(),
			}
		}

		while let Some(op) = stack.pop() {
			output.push(op);
		}

		Self { ops: output }
	}

	pub fn eval(&self, f: impl Fn(&str) -> bool) -> Option<bool> {
		let mut stack: Vec<bool> = Vec::with_capacity(self.ops.len());
		for op in &self.ops {
			match op {
				ConditionOp::Or => {
					let b = stack.pop()? | stack.pop()?;
					stack.push(b);
				}
				ConditionOp::And => {
					let b = stack.pop()? & stack.pop()?;
					stack.push(b);
				}
				ConditionOp::Not => {
					let b = !stack.pop()?;
					stack.push(b);
				}
				ConditionOp::Term(s) => {
					stack.push(f(s));
				}
				_ => return None,
			}
		}

		if stack.len() == 1 { Some(stack[0]) } else { None }
	}
}
