use yazi_core::{mgr::{CdSource, DisplaceOpt, FilterOpt, FindDoOpt, OpenDoOpt, OpenOpt, SearchOpt}, spot::SpotLock};
use yazi_macro::{emit, relay};
use yazi_shared::{Id, SStr, url::UrlBuf};
use yazi_shim::strum::IntoStr;

pub struct MgrProxy;

impl MgrProxy {
	pub fn arrow(step: impl Into<SStr>) {
		emit!(Call(relay!(mgr:arrow, [step.into()])));
	}

	pub fn cd(target: impl Into<UrlBuf>, source: CdSource) {
		emit!(Call(
			relay!(mgr:cd, [target.into()]).with("raw", true).with("source", source.into_str())
		));
	}

	pub fn displace_do(tab: Id, opt: DisplaceOpt) {
		emit!(Call(relay!(mgr:displace_do).with("tab", tab).with_any("opt", opt)));
	}

	pub fn filter_do(opt: FilterOpt) {
		emit!(Call(relay!(mgr:filter_do).with_any("opt", opt)));
	}

	pub fn find_do(opt: FindDoOpt) {
		emit!(Call(relay!(mgr:find_do).with_any("opt", opt)));
	}

	pub fn open(opt: OpenOpt) {
		emit!(Call(relay!(mgr:open).with_any("opt", opt)));
	}

	pub fn open_do(opt: OpenDoOpt) {
		emit!(Call(relay!(mgr:open_do).with_any("opt", opt)));
	}

	pub fn remove_do(targets: Vec<UrlBuf>, permanently: bool) {
		emit!(Call(
			relay!(mgr:remove_do).with("permanently", permanently).with_any("targets", targets)
		));
	}

	pub fn reveal(target: impl Into<UrlBuf>) {
		emit!(Call(relay!(mgr:reveal, [target.into()]).with("raw", true).with("no-dummy", true)));
	}

	pub fn search_do(opt: SearchOpt) {
		emit!(Call(relay!(mgr:search_do).with_any("opt", opt)));
	}

	pub fn tab_rename(tab: Id, name: impl Into<SStr>) {
		emit!(Call(relay!(mgr:tab_rename, [name.into()]).with("tab", tab)));
	}

	pub fn update_spotted(lock: SpotLock) {
		emit!(Call(relay!(mgr:update_spotted).with_any("lock", lock)));
	}
}
