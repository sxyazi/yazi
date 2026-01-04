yazi_macro::mod_pub!(bindings elements external fs isolate loader process pubsub runtime theme utils);

yazi_macro::mod_flat!(lua);

pub fn init() -> anyhow::Result<()> {
	crate::loader::init();
	crate::init_lua()?;
	Ok(())
}
