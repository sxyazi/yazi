pub fn init() -> mlua::Result<()> {
	super::active::Active::init()?;
	super::files::Files::init()?;
	super::tabs::Tabs::init()?;
	super::tasks::Tasks::init()?;

	Ok(())
}
