use mlua::Table;
use yazi_binding::{FileRef, elements::{Rect, Renderable}};
use yazi_shared::pool::{InternStr, Symbol};

#[derive(Clone, Debug, Default)]
pub struct PreviewLock {
	pub url:  yazi_shared::url::UrlBuf,
	pub cha:  yazi_fs::cha::Cha,
	pub mime: Symbol<str>,

	pub skip: usize,
	pub area: Rect,
	pub data: Vec<Renderable>,
}

impl TryFrom<Table> for PreviewLock {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		let file: FileRef = t.raw_get("file")?;
		Ok(Self {
			url:  file.url_owned(),
			cha:  file.cha,
			mime: t.raw_get::<mlua::String>("mime")?.to_str()?.intern(),

			skip: t.raw_get("skip")?,
			area: t.raw_get("area")?,
			data: Default::default(),
		})
	}
}
