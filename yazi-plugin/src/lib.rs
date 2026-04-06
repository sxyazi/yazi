yazi_macro::mod_pub!(elements external fs pubsub runtime tasks theme utils);

yazi_macro::mod_flat!(slim standard);

pub fn init() -> anyhow::Result<()> {
	LUA.init(crate::standard_lua()?);

	Ok(())
}
