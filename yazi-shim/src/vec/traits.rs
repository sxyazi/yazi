use super::IndexAtError;

pub trait VecExt<T> {
	fn index_at(&self, index: isize) -> Result<usize, IndexAtError>;

	fn insert_at(&mut self, index: isize, value: T) -> Result<(), IndexAtError>;
}

impl<T> VecExt<T> for Vec<T> {
	fn index_at(&self, index: isize) -> Result<usize, IndexAtError> {
		let len = self.len();

		if index < 0 {
			let offset = index.unsigned_abs() - 1;
			len.checked_sub(offset).ok_or(IndexAtError { index, len })
		} else if index as usize > len {
			Err(IndexAtError { index, len })
		} else {
			Ok(index as usize)
		}
	}

	fn insert_at(&mut self, index: isize, value: T) -> Result<(), IndexAtError> {
		self.insert(self.index_at(index)?, value);
		Ok(())
	}
}
