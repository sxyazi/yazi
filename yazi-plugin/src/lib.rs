#![allow(clippy::if_same_then_else, clippy::unit_arg)]

yazi_macro::mod_pub!(bindings config external fs isolate loader process pubsub utils);

yazi_macro::mod_flat!(lua twox);

pub fn init() -> anyhow::Result<()> {
	crate::loader::init();
	crate::init_lua()?;
	Ok(())
}
