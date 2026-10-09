yazi_macro::mod_pub!(entry evaluator fetcher loader preloader previewer provider spot sync);

yazi_macro::mod_flat!(coroutine runner traits);

pub static RUNNER: yazi_shim::cell::RoCell<Runner> = yazi_shim::cell::RoCell::new();

pub fn init(setter: fn(&mlua::Lua) -> mlua::Result<()>) {
	crate::loader::init();
	RUNNER.init(Runner { setter });
}
