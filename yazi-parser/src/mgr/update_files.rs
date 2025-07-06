use yazi_fs::FilesOp;
use yazi_shared::event::CmdCow;

pub struct UpdateFilesOpt {
	pub op: FilesOp,
}

impl TryFrom<CmdCow> for UpdateFilesOpt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { op: c.take_any("op").ok_or(())? })
	}
}
