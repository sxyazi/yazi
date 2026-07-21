pub trait OptionExt<T: ?Sized + ToOwned> {
	fn owned(self) -> Option<T::Owned>;
}

impl<T: ?Sized + ToOwned> OptionExt<T> for Option<&T> {
	fn owned(self) -> Option<T::Owned> { self.map(ToOwned::to_owned) }
}
