use std::io;

use mlua::{AnyUserData, IntoLuaMulti, UserData, UserDataMethods, Value};
use yazi_binding::Error;
use yazi_fs::engine::{Attrs, FileBuilder};
use yazi_shared::{auth::AuthKind, url::{AsUrl, UrlRef}};

#[derive(Clone, Copy, Default)]
pub struct Demand(yazi_fs::engine::Demand);

impl FileBuilder for Demand {
	type File = super::RwFile;

	fn append(&mut self, append: bool) -> &mut Self {
		self.0.append = append;
		self
	}

	fn attrs(&mut self, attrs: Attrs) -> &mut Self {
		self.0.attrs = attrs;
		self
	}

	fn create(&mut self, create: bool) -> &mut Self {
		self.0.create = create;
		self
	}

	fn create_new(&mut self, create_new: bool) -> &mut Self {
		self.0.create_new = create_new;
		self
	}

	async fn open<U>(&self, url: U) -> io::Result<Self::File>
	where
		U: AsUrl,
	{
		let url = url.as_url();
		Ok(match url.kind() {
			AuthKind::Regular | AuthKind::Search => {
				(self.0.build::<yazi_fs::engine::local::Demand>().open(url).await?, url.to_owned()).into()
			}
			AuthKind::Mount | AuthKind::Hub | AuthKind::Scope => {
				self.0.build::<super::lua::Demand>().open(url).await?.into()
			}
			AuthKind::Sftp => {
				(self.0.build::<super::sftp::Demand>().open(url).await?, url.to_owned()).into()
			}
		})
	}

	fn read(&mut self, read: bool) -> &mut Self {
		self.0.read = read;
		self
	}

	fn truncate(&mut self, truncate: bool) -> &mut Self {
		self.0.truncate = truncate;
		self
	}

	fn write(&mut self, write: bool) -> &mut Self {
		self.0.write = write;
		self
	}
}

impl UserData for Demand {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_function("append", |_, (ud, append): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.append(append);
			Ok(ud)
		});
		methods.add_function("create", |_, (ud, create): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.create(create);
			Ok(ud)
		});
		methods.add_function("create_new", |_, (ud, create_new): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.create_new(create_new);
			Ok(ud)
		});
		methods.add_async_method("open", |lua, me, url: UrlRef| async move {
			match me.open(&*url).await {
				Ok(fd) => fd.into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
			}
		});
		methods.add_function("read", |_, (ud, read): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.read(read);
			Ok(ud)
		});
		methods.add_function("truncate", |_, (ud, truncate): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.truncate(truncate);
			Ok(ud)
		});
		methods.add_function("write", |_, (ud, write): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.write(write);
			Ok(ud)
		});
	}
}
