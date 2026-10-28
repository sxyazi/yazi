use yazi_core::{mgr::{CdSource, DisplaceOpt, FilterOpt, FindDoOpt, OpenDoOpt, OpenOpt}, spot::SpotLock};
use yazi_macro::{emit, relay};
use yazi_shared::{scope::Scope, url::UrlBuf};
use yazi_shim::{SStr, strum::IntoStr};

pub struct MgrProxy;

impl MgrProxy {
	pub fn cd(target: impl Into<UrlBuf>, source: CdSource) {
		emit!(Call(
			relay!(mgr:cd, [target.into()]).with("raw", true).with("source", source.into_str())
		));
	}

	pub fn displace_do(scope: impl Into<Scope>, opt: DisplaceOpt) {
		emit!(Call(relay!(mgr:displace_do).with_scope(scope).with_any("opt", opt)));
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

	pub fn remove_do(permanently: bool, targets: impl Into<Vec<UrlBuf>>) {
		emit!(Call(
			relay!(mgr:remove_do).with("permanently", permanently).with_any("targets", targets.into())
		));
	}

	pub fn reveal(target: impl Into<UrlBuf>) {
		emit!(Call(relay!(mgr:reveal, [target.into()]).with("raw", true).with("no-dummy", true)));
	}

	pub fn tab_rename(scope: impl Into<Scope>, name: impl Into<SStr>) {
		emit!(Call(relay!(mgr:tab_rename, [name.into()]).with_scope(scope)));
	}

	pub fn update_spotted(lock: SpotLock) {
		emit!(Call(relay!(mgr:update_spotted).with_scope(&lock).with_any("lock", lock)));
	}
}
