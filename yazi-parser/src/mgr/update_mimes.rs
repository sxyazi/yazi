use std::collections::HashMap;

use yazi_shared::event::{CmdCow, Data, DataKey};

pub struct UpdateMimesOpt {
	pub updates: HashMap<DataKey, Data>,
}

impl TryFrom<CmdCow> for UpdateMimesOpt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { updates: c.try_take("updates").and_then(Data::into_dict).ok_or(())? })
	}
}
