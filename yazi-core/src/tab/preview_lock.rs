use mlua::{LuaString, Table};
use yazi_binding::elements::Rect;
use yazi_fs::{cha::Cha, file::FileRef};
use yazi_macro::impl_data_any;
use yazi_shared::{pool::{InternStr, Symbol}, url::UrlBuf};
use yazi_widgets::Renderable;

#[derive(Clone, Debug, Default)]
pub struct PreviewLock {
	pub url:  UrlBuf,
	pub cha:  Cha,
	pub mime: Symbol<str>,

	pub skip: usize,
	pub area: Rect,
	pub data: Vec<Renderable>,
}

impl_data_any!(PreviewLock);

impl TryFrom<Table> for PreviewLock {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		let file: FileRef = t.raw_get("file")?;
		file.borrow(|f| {
			Ok(Self {
				url:  f.url_owned(),
				cha:  f.cha,
				mime: t.raw_get::<LuaString>("mime")?.to_str()?.intern(),

				skip: t.raw_get("skip")?,
				area: t.raw_get("area")?,
				data: Default::default(),
			})
		})
	}
}
