yazi_macro::mod_pub!(entry fetcher loader preloader previewer);

yazi_macro::mod_flat!(runner spot);

pub static RUNNER: yazi_shim::cell::RoCell<Runner> = yazi_shim::cell::RoCell::new();

pub fn init(setter: fn(&mlua::Lua) -> mlua::Result<()>) {
	crate::loader::init();

	RUNNER.init(Runner { setter });
}
