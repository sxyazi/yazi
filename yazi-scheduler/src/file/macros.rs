macro_rules! impl_from_in {
	($($variant:ident($type:ty)),* $(,)?) => {
		$(
			impl From<$type> for $crate::file::FileIn {
				fn from(value: $type) -> Self { Self::$variant(value) }
			}
		)*
	};
}
