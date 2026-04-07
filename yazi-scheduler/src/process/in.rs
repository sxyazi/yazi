use std::{borrow::Cow, ffi::OsString};

use yazi_shared::{CompletionToken, Id, url::UrlCow};

use super::ShellOpt;
use crate::{TaskIn, process::{ProcessProgBg, ProcessProgBlock, ProcessProgOrphan}};

#[derive(Debug)]
pub(crate) enum ProcessIn {
	Block(ProcessInBlock),
	Orphan(ProcessInOrphan),
	Bg(ProcessInBg),
}

impl_from_in!(Block(ProcessInBlock), Orphan(ProcessInOrphan), Bg(ProcessInBg));

impl TaskIn for ProcessIn {
	type Prog = ();

	fn id(&self) -> Id {
		match self {
			Self::Block(r#in) => r#in.id,
			Self::Orphan(r#in) => r#in.id,
			Self::Bg(r#in) => r#in.id,
		}
	}

	fn set_id(&mut self, id: Id) -> &mut Self {
		match self {
			Self::Block(r#in) => _ = r#in.set_id(id),
			Self::Orphan(r#in) => _ = r#in.set_id(id),
			Self::Bg(r#in) => _ = r#in.set_id(id),
		};
		self
	}

	fn title(&self) -> Cow<'_, str> {
		match self {
			Self::Block(r#in) => r#in.title(),
			Self::Orphan(r#in) => r#in.title(),
			Self::Bg(r#in) => r#in.title(),
		}
	}
}

// --- Block
#[derive(Debug)]
pub(crate) struct ProcessInBlock {
	pub(crate) id:   Id,
	pub(crate) cwd:  UrlCow<'static>,
	pub(crate) cmd:  OsString,
	pub(crate) args: Vec<UrlCow<'static>>,
}

impl TaskIn for ProcessInBlock {
	type Prog = ProcessProgBlock;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> { format!("Blocking command: {}", self.cmd.display()).into() }
}

impl From<ProcessInBlock> for ShellOpt {
	fn from(r#in: ProcessInBlock) -> Self {
		Self { cwd: r#in.cwd, cmd: r#in.cmd, args: r#in.args, piped: false, orphan: false }
	}
}

// --- Orphan
#[derive(Debug)]
pub(crate) struct ProcessInOrphan {
	pub(crate) id:   Id,
	pub(crate) cwd:  UrlCow<'static>,
	pub(crate) cmd:  OsString,
	pub(crate) args: Vec<UrlCow<'static>>,
}

impl TaskIn for ProcessInOrphan {
	type Prog = ProcessProgOrphan;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> { format!("Orphan command: {}", self.cmd.display()).into() }
}

impl From<ProcessInOrphan> for ShellOpt {
	fn from(r#in: ProcessInOrphan) -> Self {
		Self { cwd: r#in.cwd, cmd: r#in.cmd, args: r#in.args, piped: false, orphan: true }
	}
}

// --- Bg
#[derive(Debug)]
pub(crate) struct ProcessInBg {
	pub(crate) id:   Id,
	pub(crate) cwd:  UrlCow<'static>,
	pub(crate) cmd:  OsString,
	pub(crate) args: Vec<UrlCow<'static>>,
	pub(crate) done: CompletionToken,
}

impl TaskIn for ProcessInBg {
	type Prog = ProcessProgBg;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> { format!("Background command: {}", self.cmd.display()).into() }
}

impl From<ProcessInBg> for ShellOpt {
	fn from(r#in: ProcessInBg) -> Self {
		Self { cwd: r#in.cwd, cmd: r#in.cmd, args: r#in.args, piped: true, orphan: false }
	}
}
