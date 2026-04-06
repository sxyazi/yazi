use mlua::{UserData, UserDataMethods};
use yazi_proxy::TasksProxy;

use crate::tasks::Task;

#[derive(Clone, Debug)]
pub(crate) struct TaskOpt(pub(crate) yazi_core::tasks::TaskOpt);

impl UserData for TaskOpt {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_async_method_once("spawn", |_, me, ()| async move {
			Ok(Task { id: TasksProxy::spawn(me.0).await? })
		});
	}
}
