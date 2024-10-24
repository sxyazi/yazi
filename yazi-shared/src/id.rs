use std::{fmt::Display, str::FromStr, sync::atomic::{AtomicUsize, Ordering}};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Id(usize);

impl Id {
	#[inline]
	pub fn as_usize(&self) -> usize { self.0 }
}

impl Display for Id {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.0.fmt(f) }
}

impl FromStr for Id {
	type Err = <usize as FromStr>::Err;

	fn from_str(s: &str) -> Result<Self, Self::Err> { s.parse().map(Self) }
}

impl TryFrom<i64> for Id {
	type Error = <usize as TryFrom<i64>>::Error;

	fn try_from(value: i64) -> Result<Self, Self::Error> { usize::try_from(value).map(Self) }
}

// --- Ids
pub struct Ids {
	next: AtomicUsize,
}

impl Ids {
	#[inline]
	pub const fn new() -> Self { Self { next: AtomicUsize::new(1) } }

	#[inline]
	pub fn next(&self) -> Id {
		loop {
			let old = self.next.fetch_add(1, Ordering::Relaxed);
			if old != 0 {
				return Id(old);
			}
		}
	}

	#[inline]
	pub fn current(&self) -> Id { Id(self.next.load(Ordering::Relaxed)) }
}

impl Default for Ids {
	fn default() -> Self { Self::new() }
}
