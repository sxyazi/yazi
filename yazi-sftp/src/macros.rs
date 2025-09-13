#[macro_export]
macro_rules! impl_from_packet {
	($($variant:ident($type:ty)),* $(,)?) => {
		$(
			impl<'a> From<$type> for $crate::Packet<'a> {
				fn from(value: $type) -> Self {
					Self::$variant(value)
				}
			}
		)*
	};
}

#[macro_export]
macro_rules! impl_try_from_packet {
	($($variant:ident($type:ty)),* $(,)?) => {
		$(
			impl<'a> TryFrom<$crate::Packet<'a>> for $type {
				type Error = $crate::Error;

				fn try_from(value: $crate::Packet<'a>) -> Result<Self, Self::Error> {
					match value {
						$crate::Packet::$variant(v) => Ok(v),
						_ => Err($crate::Error::Packet(concat!("not a ", stringify!($variant)))),
					}
				}
			}
		)*
	};
}
