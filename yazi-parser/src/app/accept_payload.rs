use anyhow::bail;
use yazi_dds::Payload;
use yazi_shared::event::CmdCow;

pub struct AcceptPayload {
	pub payload: Payload<'static>,
}

impl TryFrom<CmdCow> for AcceptPayload {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		let Some(payload) = c.take_any("payload") else {
			bail!("Invalid 'payload' in AcceptPayload");
		};

		Ok(Self { payload })
	}
}
