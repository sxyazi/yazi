use std::{ops::{Deref, DerefMut}, sync::Arc};

use mlua::{BorrowedBytes, ExternalError, FromLua, IntoLua, IntoLuaMulti, MetaMethod, UserData, UserDataMethods, Value};

use super::{Domains, DomainsMatcher, Service};

#[derive(Clone, Debug)]
pub struct DomainsArc(Arc<Domains>);

impl Deref for DomainsArc {
	type Target = Arc<Domains>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for DomainsArc {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl From<Domains> for DomainsArc {
	fn from(value: Domains) -> Self { Self(value.into()) }
}

impl DomainsArc {
	pub(crate) fn unwrap_unchecked(self) -> Domains {
		Arc::try_unwrap(self.0).expect("unique domains arc")
	}
}

impl UserData for DomainsArc {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Index, |lua, me, domain: BorrowedBytes| {
			me.get(&domain).into_lua(lua)
		});

		methods.add_meta_method(
			MetaMethod::NewIndex,
			|lua, me, (domain, value): (BorrowedBytes, Value)| {
				match value {
					t @ Value::Table(_) => {
						let mut service = Service::from_lua(t, lua)?;
						service.configure(&me.scheme, &domain).map_err(|e| e.into_lua_err())?;
						me.insert(&domain, service);
					}
					Value::Nil => me.remove(&domain),
					_ => return Err("expected a table or nil".into_lua_err()),
				}
				Ok(())
			},
		);

		methods.add_meta_method(MetaMethod::Pairs, |lua, me, ()| {
			let mut matcher = DomainsMatcher::from(me);
			let iter = lua.create_function_mut(move |lua, ()| {
				if let Some((domain, service)) = matcher.next() {
					(domain, service).into_lua_multi(lua)
				} else {
					().into_lua_multi(lua)
				}
			})?;
			Ok(iter)
		});
	}
}
