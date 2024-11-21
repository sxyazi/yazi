#![allow(clippy::unit_arg)]

mod macros;

yazi_macro::mod_pub!(
	bindings, elements, external, file, fs, isolate, loader, process, pubsub, url, utils
);

yazi_macro::mod_flat!(clipboard config error lua runtime);

pub fn init() -> anyhow::Result<()> {
	CLIPBOARD.with(<_>::default);

	crate::loader::init();
	crate::init_lua()?;
	Ok(())
}
