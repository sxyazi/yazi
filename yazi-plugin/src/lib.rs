yazi_macro::mod_pub!(fs keymap pubsub runtime tasks theme ui utils);

yazi_macro::mod_flat!(scope_ext slim standard);

pub async fn setup() -> anyhow::Result<()> {
	LUA.init(crate::standard_lua().await?);

	Ok(())
}

pub fn shutdown() { utils::shutdown() }
