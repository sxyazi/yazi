use yazi_shared::{event::{ActionCow, FromAction}, strand::StrandBuf};

#[derive(Clone, Debug)]
pub struct CmpItem {
	pub name:   StrandBuf,
	pub value:  Option<String>,
	pub is_dir: bool,
}

impl<C> FromAction<C> for CmpItem {
	fn from_action(mut a: ActionCow, _: &C) -> anyhow::Result<Self> {
		Ok(Self { name: a.take("name")?, value: a.take("value").ok(), is_dir: a.bool("is_dir") })
	}
}
