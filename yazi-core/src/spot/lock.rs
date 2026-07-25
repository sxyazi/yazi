use mlua::Table;
use yazi_fs::{cha::Cha, file::FileRef};
use yazi_macro::impl_data_any;
use yazi_shared::{id::Id, url::UrlBuf};
use yazi_widgets::Renderable;

#[derive(Clone, Debug)]
pub struct SpotLock {
	pub url:  UrlBuf,
	pub cha:  Cha,
	pub mime: String,

	pub id:   Id,
	pub skip: usize,
	pub data: Vec<Renderable>,
}

impl_data_any!(SpotLock);

impl TryFrom<Table> for SpotLock {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		let file: FileRef = t.raw_get("file")?;
		file.borrow(|f| {
			Ok(Self {
				url:  f.url_owned(),
				cha:  f.cha,
				mime: t.raw_get("mime")?,

				id:   t.raw_get("id")?,
				skip: t.raw_get("skip")?,
				data: Default::default(),
			})
		})
	}
}

impl SpotLock {
	pub fn len(&self) -> Option<usize> { Some(self.table()?.len()) }

	pub fn select(&mut self, idx: Option<usize>) {
		if let Some(t) = self.table_mut() {
			t.select(idx);
		}
	}

	pub fn selected(&self) -> Option<usize> { self.table()?.selected() }

	pub fn table(&self) -> Option<&yazi_binding::elements::Table> {
		self.data.iter().rev().find_map(|r| match r {
			Renderable::Table(t) => Some(t.as_ref()),
			_ => None,
		})
	}

	pub fn table_mut(&mut self) -> Option<&mut yazi_binding::elements::Table> {
		self.data.iter_mut().rev().find_map(|r| match r {
			Renderable::Table(t) => Some(t.as_mut()),
			_ => None,
		})
	}
}
