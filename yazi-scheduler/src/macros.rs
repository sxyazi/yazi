#[macro_export]
macro_rules! impl_from_in {
	($($variant:ident($type:ty)),* $(,)?) => {
		$(
			impl From<$type> for $crate::TaskIn {
				fn from(value: $type) -> Self { Self::$variant(value) }
			}
		)*
	};
}

#[macro_export]
macro_rules! impl_from_out {
	($($variant:ident($type:ty)),* $(,)?) => {
		$(
			impl From<$type> for $crate::TaskOut {
				fn from(value: $type) -> Self { Self::$variant(value) }
			}
		)*
	};
}

#[macro_export]
macro_rules! impl_from_prog {
	($($variant:ident($type:ty)),* $(,)?) => {
		$(
			impl From<$type> for $crate::TaskProg {
				fn from(value: $type) -> Self { Self::$variant(value) }
			}
		)*
	};
}
