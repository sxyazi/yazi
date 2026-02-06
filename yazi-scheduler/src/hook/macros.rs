macro_rules! impl_from_in {
	($($variant:ident($type:ty)),* $(,)?) => {
		$(
			impl From<$type> for $crate::hook::HookIn {
				fn from(value: $type) -> Self { Self::$variant(value) }
			}
		)*
	};
}
