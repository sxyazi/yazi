pub fn init() -> mlua::Result<()> {
	super::tab::Tab::init()?;
	super::tasks::Tasks::init()?;

	Ok(())
}
