use std::{fmt::{self, Display}, io::{self, Write}};

use base64::{Engine, write::EncoderStringWriter};

/// Types that can be iterated to produce a list of MIME types.
pub(super) trait Mimelist: IntoIterator<Item: Display> + Clone {}

impl<T> Mimelist for T
where
	T: IntoIterator + Clone,
	T::Item: Display,
{
}

// --- ListMimes
pub(super) struct ListMimes<M>(pub M);

impl<M: Mimelist> ListMimes<M> {
	pub(super) fn encode_base64<E: Engine>(&self, engine: &E) -> io::Result<String> {
		let mut writer = EncoderStringWriter::new(engine);
		write!(writer, "{self}")?;
		Ok(writer.into_inner())
	}
}

impl<M: Mimelist> Display for ListMimes<M> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for (i, mime) in self.0.clone().into_iter().enumerate() {
			if i > 0 {
				f.write_str(" ")?;
			}
			write!(f, "{mime}")?;
		}
		Ok(())
	}
}
