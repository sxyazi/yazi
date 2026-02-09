use mlua::{AnyUserData, IntoLuaMulti, UserData, UserDataMethods, Value};
use yazi_fs::provider::FileBuilder;

use crate::{Error, Fd, UrlRef};

#[derive(Default)]
pub struct Access(yazi_vfs::provider::Gate);

impl UserData for Access {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_function_mut("append", |_, (ud, append): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.0.append(append);
			Ok(ud)
		});
		methods.add_function_mut("create", |_, (ud, create): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.0.create(create);
			Ok(ud)
		});
		methods.add_function_mut("create_new", |_, (ud, create_new): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.0.create_new(create_new);
			Ok(ud)
		});
		methods.add_async_method("open", |lua, me, url: UrlRef| async move {
			match me.0.open(&*url).await {
				Ok(fd) => Fd(fd).into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Io(e)).into_lua_multi(&lua),
			}
		});
		methods.add_function_mut("read", |_, (ud, read): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.0.read(read);
			Ok(ud)
		});
		methods.add_function_mut("truncate", |_, (ud, truncate): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.0.truncate(truncate);
			Ok(ud)
		});
		methods.add_function_mut("write", |_, (ud, write): (AnyUserData, bool)| {
			ud.borrow_mut::<Self>()?.0.write(write);
			Ok(ud)
		});
	}
}
