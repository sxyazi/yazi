use mlua::UserData;

use crate::data::DataAny;

#[derive(Debug, UserData)]
pub struct AnyData(pub Box<dyn DataAny>);
