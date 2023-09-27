pub fn init() -> mlua::Result<()> {
	super::manager::Manager::init()?;
	super::tasks::Tasks::init()?;

	Ok(())
}
