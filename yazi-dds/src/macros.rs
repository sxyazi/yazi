#[macro_export]
macro_rules! spark {
	(mgr: $name:ident, $body:expr) => {
		paste::paste! {
			$crate::spark::Spark::[<$name:camel>]($body)
		}
	};
	($layer:ident : $name:ident, $body:expr) => {
		paste::paste! {
			$crate::spark::Spark::[<$layer:camel $name:camel>]($body.into())
		}
	};
}

#[macro_export]
macro_rules! try_from_spark {
	($opt:ty, $($($layer:ident)? : $name:ident),+) => {
		impl<'a> std::convert::TryFrom<$crate::spark::Spark<'a>> for paste::paste! { yazi_parser::$opt } {
			type Error = ();

			fn try_from(value: $crate::spark::Spark<'a>) -> Result<Self, Self::Error> {
				$(
					try_from_spark!(@if $($layer)? : $name, value);
				)+
				Err(())
			}
		}
	};
	(@if mgr : $name:ident, $value:ident) => {
		try_from_spark!(@if : $name, $value);
	};
	(@if $($layer:ident)? : $name:ident, $value:ident) => {
		if let paste::paste! { $crate::spark::Spark::[<$($layer:camel)* $name:camel>](opt) } = $value {
			return Ok(<_>::from(opt))
		}
	};
}
