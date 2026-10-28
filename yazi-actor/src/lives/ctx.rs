use std::ops::Deref;

use mlua::{AnyUserData, BorrowedBytes, IntoLua, Lua, MetaMethod, UserData, UserDataMethods, UserDataRef, Value};
use paste::paste;
use yazi_binding::runtime;
use yazi_shared::scope::ScopeLike;

use super::{Lives, PtrCell};

pub(super) type CtxRef = UserDataRef<Ctx>;

pub(super) struct Ctx {
	inner: PtrCell<yazi_core::Ctx<'static>>,

	c_tab:    Option<Value>,
	c_tabs:   Option<Value>,
	c_tasks:  Option<Value>,
	c_yanked: Option<Value>,
	c_input:  Option<Value>,
	c_which:  Option<Value>,
	c_layer:  Option<Value>,
}

impl Deref for Ctx {
	type Target = yazi_core::Ctx<'static>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Ctx {
	pub(super) fn make(cx: &yazi_core::Ctx) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self {
			inner: PtrCell::from(cx).cast(),

			c_tab:    None,
			c_tabs:   None,
			c_tasks:  None,
			c_yanked: None,
			c_input:  None,
			c_which:  None,
			c_layer:  None,
		})
	}

	fn tab(&self, lua: &Lua) -> mlua::Result<Option<AnyUserData>> {
		self
			.mgr
			.tabs
			.idx(runtime!(lua)?.scope().tab)
			.map(|idx| super::Tab::make(idx, &self.mgr.tabs[idx]))
			.transpose()
	}
}

impl UserData for Ctx {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method_mut(MetaMethod::Index, |lua, me, key: BorrowedBytes| {
			macro_rules! reuse {
				($key:ident, $value:expr) => {
					match paste! { &me.[<c_ $key>] } {
						Some(v) => v.clone(),
						None => {
							let v = $value?.into_lua(lua)?;
							paste! { me.[<c_ $key>] = Some(v.clone()); };
							v
						}
					}
				};
			}

			Ok(match &*key {
				b"active" => super::Tab::make(me.mgr.tabs.cursor, me.active())?.into_lua(lua)?,
				b"tab" => reuse!(tab, me.tab(lua)),
				b"tabs" => reuse!(tabs, super::Tabs::make(&me.mgr.tabs)),
				b"tasks" => reuse!(tasks, super::Tasks::make(&me.tasks)),
				b"yanked" => reuse!(yanked, super::Yanked::make(&me.mgr.yanked)),
				b"input" => reuse!(input, super::Input::make(&me.input)),
				b"which" => reuse!(which, super::Which::make(&me.which)),
				b"layer" => reuse!(layer, Ok::<_, mlua::Error>(me.layer())),
				_ => Value::Nil,
			})
		});
	}
}
