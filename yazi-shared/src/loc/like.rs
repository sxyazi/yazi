use super::{LocBuf, LocBufAble, LocBufAbleImpl};

pub trait LocLike {
	type Borrowed<'a>
	where
		Self: 'a;

	fn base(&self) -> Self::Borrowed<'_>;

	fn trail(&self) -> Self::Borrowed<'_>;

	fn uri(&self) -> Self::Borrowed<'_>;

	fn urn(&self) -> Self::Borrowed<'_>;
}

impl<P> LocLike for LocBuf<P>
where
	P: LocBufAble + LocBufAbleImpl,
{
	type Borrowed<'a>
		= P::Borrowed<'a>
	where
		Self: 'a;

	fn base(&self) -> Self::Borrowed<'_> { self.as_loc().base() }

	fn trail(&self) -> Self::Borrowed<'_> { self.as_loc().trail() }

	fn uri(&self) -> Self::Borrowed<'_> { self.as_loc().uri() }

	fn urn(&self) -> Self::Borrowed<'_> { self.as_loc().urn() }
}
