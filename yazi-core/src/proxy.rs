use mlua::ObjectLike;
use yazi_config::LAYOUT;
use yazi_macro::{emit, relay};
use yazi_runner::previewer::PeekJob;
use yazi_scheduler::TaskSummary;
use yazi_shared::url::AsUrl;
use yazi_shim::fs::Error;
use yazi_widgets::Renderable;

use crate::{app::PluginOpt, tab::PreviewLock};

pub struct AppProxy;

impl AppProxy {
	pub fn plugin_do(opt: PluginOpt) {
		emit!(Call(relay!(app:plugin_do).with_any("opt", opt)));
	}

	pub(crate) fn plugin_peek(job: PeekJob) {
		Self::plugin_do(PluginOpt::new_callback(job, |_, plugin, job| plugin.call_method("peek", job)));
	}

	pub(crate) fn update_progress(summary: TaskSummary) {
		emit!(Call(relay!(app:update_progress).with_any("summary", summary)));
	}
}

// --- Mgr
pub struct MgrProxy;

impl MgrProxy {
	pub(crate) fn update_paged_by<U>(page: usize, only_if: U)
	where
		U: AsUrl,
	{
		emit!(Call(relay!(mgr:update_paged, [page]).with("only-if", only_if.as_url())));
	}

	pub fn update_peeked(lock: PreviewLock) {
		emit!(Call(relay!(mgr:update_peeked).with_scope(&lock).with_any("lock", lock)));
	}

	pub(crate) fn update_peeked_error(job: PeekJob, error: String) {
		let area = LAYOUT.get().preview;
		Self::update_peeked(PreviewLock {
			url:   job.file.url,
			sig:   job.sig,
			scope: job.scope,

			skip: job.skip,
			area: area.into(),
			data: vec![
				Renderable::Clear(Default::default()).with_area(area),
				Renderable::from(Error::other(error)).with_area(area),
			],
		});
	}
}
