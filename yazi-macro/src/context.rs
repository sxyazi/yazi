#[macro_export]
macro_rules! tab {
	($cx:ident) => {{
		let tab = $cx.tab;
		&mut $cx.core.mgr.tabs.items[tab]
	}};
}
