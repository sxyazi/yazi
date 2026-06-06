use mlua::UserData;

use crate::data::DataAny;

#[derive(Debug)]
pub struct AnyData(pub Box<dyn DataAny>);

impl UserData for AnyData {}
