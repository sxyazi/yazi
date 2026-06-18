use std::fmt::Display;

/// Types that can be iterated to produce a list of MIME types.
pub(super) trait Mimelist: IntoIterator<Item: Display> + Clone {}

impl<T> Mimelist for T
where
	T: IntoIterator + Clone,
	T::Item: Display,
{
}
