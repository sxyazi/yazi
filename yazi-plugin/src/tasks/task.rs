use std::ops::Deref;

use mlua::{LuaString, Table, UserData, UserDataFields, UserDataMethods};
use yazi_proxy::TasksProxy;
use yazi_scheduler::{TaskHandle, custom::CustomOut};
use yazi_shared::url::UrlBuf;

#[derive(Clone, Debug)]
pub(crate) struct Task(pub(super) TaskHandle);

impl Deref for Task {
	type Target = TaskHandle;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl UserData for Task {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(me.id));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method("acquire", |_, me, ()| async move { Ok(me.started().await) });
		methods.add_method("progress", |_, me, t: Table| {
			TasksProxy::output(me.id, CustomOut::Progress {
				total:     t.raw_get::<Option<_>>("total")?.unwrap_or_default(),
				success:   t.raw_get::<Option<_>>("success")?.unwrap_or_default(),
				failed:    t.raw_get::<Option<_>>("failed")?.unwrap_or_default(),
				workload:  t.raw_get::<Option<_>>("workload")?.unwrap_or_default(),
				processed: t.raw_get::<Option<_>>("processed")?.unwrap_or_default(),
			});
			Ok(())
		});
		methods.add_method("log", |_, me, line: LuaString| {
			TasksProxy::output(me.id, CustomOut::Log(line.to_string_lossy()));
			Ok(())
		});
		methods.add_method("succeed", |_, me, urls: Option<Vec<UrlBuf>>| {
			TasksProxy::output(me.id, CustomOut::Succ(urls.unwrap_or_default()));
			Ok(())
		});
		methods.add_method("fail", |_, me, reason: LuaString| {
			TasksProxy::output(me.id, CustomOut::Fail(reason.to_string_lossy()));
			Ok(())
		});
	}
}
