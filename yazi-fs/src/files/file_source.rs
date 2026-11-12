use crate::file::File;

pub trait FileSource {
	fn len(&self) -> usize;

	fn borrow(
		&self,
		index: usize,
		visit: &mut dyn FnMut(&File) -> mlua::Result<()>,
	) -> mlua::Result<()>;
}
