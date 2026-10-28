use super::Scope;

pub trait ScopeLike {
	fn scope(&self) -> &Scope;

	fn to_scope(&self) -> Scope { self.into() }
}

impl<T: ScopeLike + ?Sized> From<&T> for Scope {
	fn from(value: &T) -> Self { value.scope().clone() }
}
