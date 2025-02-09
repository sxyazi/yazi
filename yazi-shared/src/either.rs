#[derive(Clone, Copy, Debug)]
pub enum Either<L, R> {
	Left(L),
	Right(R),
}

impl<L, R> Either<L, R> {
	pub fn left(&self) -> Option<&L> {
		match self {
			Either::Left(l) => Some(l),
			_ => None,
		}
	}

	pub fn right(&self) -> Option<&R> {
		match self {
			Either::Right(r) => Some(r),
			_ => None,
		}
	}

	pub fn left_mut(&mut self) -> Option<&mut L> {
		match self {
			Either::Left(l) => Some(l),
			_ => None,
		}
	}

	pub fn right_mut(&mut self) -> Option<&mut R> {
		match self {
			Either::Right(r) => Some(r),
			_ => None,
		}
	}

	pub fn is_left_and<F: FnOnce(&L) -> bool>(&self, f: F) -> bool {
		self.left().map(f).unwrap_or(false)
	}

	pub fn is_right_and<F: FnOnce(&R) -> bool>(&self, f: F) -> bool {
		self.right().map(f).unwrap_or(false)
	}

	pub fn into_left(self) -> Option<L> {
		match self {
			Either::Left(l) => Some(l),
			_ => None,
		}
	}

	pub fn into_right(self) -> Option<R> {
		match self {
			Either::Right(r) => Some(r),
			_ => None,
		}
	}
}
