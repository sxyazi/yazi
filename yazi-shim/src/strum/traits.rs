pub trait IntoStr {
	fn into_str(self) -> &'static str;
}

impl<T> IntoStr for T
where
	T: Into<&'static str>,
{
	fn into_str(self) -> &'static str { self.into() }
}
