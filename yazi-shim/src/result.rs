pub trait ResultExt<T: ?Sized + ToOwned, E> {
	fn owned(self) -> Result<T::Owned, E>;
}

impl<T: ?Sized + ToOwned, E> ResultExt<T, E> for Result<&T, E> {
	fn owned(self) -> Result<T::Owned, E> { self.map(ToOwned::to_owned) }
}
