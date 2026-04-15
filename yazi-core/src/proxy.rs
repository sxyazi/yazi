use mlua::ObjectLike;
use yazi_binding::{Error, elements::Renderable};
use yazi_config::LAYOUT;
use yazi_macro::{emit, relay};
use yazi_runner::previewer::PeekJob;
use yazi_scheduler::TaskSummary;
use yazi_shared::url::AsUrl;

use crate::{app::PluginOpt, tab::PreviewLock};

pub struct AppProxy;

impl AppProxy {
	pub fn plugin(opt: PluginOpt) {
		emit!(Call(relay!(app:plugin).with_any("opt", opt)));
	}

	pub fn plugin_peek(job: PeekJob) {
		let name = job.previewer.name.clone();
		Self::plugin(PluginOpt::new_callback(name, move |_, plugin| plugin.call_method("peek", job)));
	}

	pub fn update_progress(summary: TaskSummary) {
		emit!(Call(relay!(app:update_progress).with_any("summary", summary)));
	}
}

// --- Mgr
pub struct MgrProxy;

impl MgrProxy {
	pub fn update_paged_by<U>(page: usize, only_if: U)
	where
		U: AsUrl,
	{
		emit!(Call(relay!(mgr:update_paged, [page]).with("only-if", only_if.as_url())));
	}

	pub fn update_peeked(lock: PreviewLock) {
		emit!(Call(relay!(mgr:update_peeked).with_any("lock", lock)));
	}

	pub fn update_peeked_error(job: PeekJob, error: String) {
		let area = LAYOUT.get().preview;
		Self::update_peeked(PreviewLock {
			url:  job.file.url,
			cha:  job.file.cha,
			mime: job.mime,

			skip: job.skip,
			area: area.into(),
			data: vec![
				Renderable::Clear(yazi_binding::elements::Clear { area: area.into() }),
				Renderable::from(Error::custom(error)).with_area(area),
			],
		});
	}
}
