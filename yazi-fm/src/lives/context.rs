use std::ops::Deref;

use mlua::{AnyUserData, IntoLua, MetaMethod, UserData, Value};
use paste::paste;

use super::{Lives, PtrCell};

pub(super) struct Ctx {
	inner: PtrCell<crate::Ctx>,

	c_active: Option<Value>,
	c_tabs:   Option<Value>,
	c_tasks:  Option<Value>,
	c_yanked: Option<Value>,
	c_layer:  Option<Value>,
}

impl Deref for Ctx {
	type Target = crate::Ctx;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl Ctx {
	#[inline]
	pub(super) fn make(inner: &crate::Ctx) -> mlua::Result<AnyUserData> {
		Lives::scoped_userdata(Self {
			inner: inner.into(),

			c_active: None,
			c_tabs:   None,
			c_tasks:  None,
			c_yanked: None,
			c_layer:  None,
		})
	}
}

impl UserData for Ctx {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method_mut(MetaMethod::Index, |lua, me, key: mlua::String| {
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
			Ok(match key.as_bytes().as_ref() {
				b"active" => reuse!(active, super::Tab::make(me.active())),
				b"tabs" => reuse!(tabs, super::Tabs::make(&me.mgr.tabs)),
				b"tasks" => reuse!(tasks, super::Tasks::make(&me.tasks)),
				b"yanked" => reuse!(yanked, super::Yanked::make(&me.mgr.yanked)),
				b"layer" => {
					reuse!(layer, Ok::<_, mlua::Error>(yazi_plugin::bindings::Layer::from(me.layer())))
				}
				_ => Value::Nil,
			})
		});
	}
}
