use mlua::UserData;

// Url
pub struct Url(shared::Url);

impl From<&shared::Url> for Url {
	fn from(value: &shared::Url) -> Self { Self(value.clone()) }
}

// TODO
impl UserData for Url {}
