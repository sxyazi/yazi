use anyhow::bail;

pub(crate) fn float_to_i64<T>(value: T) -> anyhow::Result<i64>
where
	T: Into<f64>,
{
	let value = value.into();
	if !value.is_finite() || value.fract() != 0.0 {
		bail!("not an integer");
	}

	let integer = value as i64;
	if integer as f64 != value {
		bail!("not an integer");
	}

	Ok(integer)
}

macro_rules! ref_or_owned {
	(ref, $value:expr) => {
		&$value
	};
	(owned, $value:expr) => {
		$value
	};
}

macro_rules! impl_into_integer {
	($a:ty, $($b:ty),+ $(,)?) => {
		impl_into_integer!(@impl ref   $a,  $a, $($b),+);
		impl_into_integer!(@impl owned $a, &$a, $($b),+);
	};
	(@impl $kind:ident $t:ty, $a:ty, $($b:ty),+ $(,)?) => {
		$(
			impl TryFrom<$a> for $b {
				type Error = anyhow::Error;

				fn try_from(value: $a) -> Result<Self, Self::Error> {
					paste::paste! {
						Ok(match ref_or_owned!($kind, value) {
							$t::Integer(i) => <$b>::try_from(*i)?,
							$t::Number(n) => <$b>::try_from($crate::data::macros::float_to_i64(*n)?)?,
							$t::String(s) => s.parse()?,
							$t::Id(i) => <$b>::try_from(i.get())?,
							_ => anyhow::bail!("not an integer"),
						})
					}
				}
			}
		)+
	};
}

macro_rules! impl_into_number {
	($a:ty, $($b:ty),+ $(,)?) => {
		impl_into_number!(@impl ref   $a,  $a, $($b),+);
		impl_into_number!(@impl owned $a, &$a, $($b),+);
	};
	(@impl $kind:ident $t:ty, $a:ty, $($b:ty),+ $(,)?) => {
		$(
			impl TryFrom<$a> for $b {
				type Error = anyhow::Error;

				fn try_from(value: $a) -> Result<Self, Self::Error> {
					paste::paste! {
						Ok(match ref_or_owned!($kind, value) {
							$t::Integer(i) if *i == (*i as $b as _) => *i as $b,
							$t::Number(n) if f64::from(*n) == (f64::from(*n) as $b as _) => f64::from(*n) as $b,
							$t::String(s) => s.parse()?,
							$t::Id(i) if i.get() == (i.get() as $b as _) => i.get() as $b,
							_ => anyhow::bail!("not a number"),
						})
					}
				}
			}
		)+
	};
}
