#![allow(clippy::unit_arg)]

mod macros;

yazi_macro::mod_pub!(bindings config elements external file fs isolate loader process pubsub utils);

yazi_macro::mod_flat!(clipboard composer id lua runtime);

pub fn init() -> anyhow::Result<()> {
	CLIPBOARD.with(<_>::default);

	crate::loader::init();
	crate::init_lua()?;
	Ok(())
}
