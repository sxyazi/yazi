yazi_macro::mod_pub!(entry fetcher loader preloader previewer);

yazi_macro::mod_flat!(runner spot);

pub static RUNNER: yazi_shared::RoCell<Runner> = yazi_shared::RoCell::new();

pub fn init(setter: fn(&mlua::Lua) -> mlua::Result<()>) {
	crate::loader::init();

	RUNNER.init(Runner { setter });
}
