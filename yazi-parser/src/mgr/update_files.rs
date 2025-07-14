use anyhow::bail;
use yazi_fs::FilesOp;
use yazi_shared::event::CmdCow;

pub struct UpdateFilesOpt {
	pub op: FilesOp,
}

impl TryFrom<CmdCow> for UpdateFilesOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(op) = c.take_any("op") else {
			bail!("Invalid 'op' argument in UpdateFilesOpt");
		};

		Ok(Self { op })
	}
}
