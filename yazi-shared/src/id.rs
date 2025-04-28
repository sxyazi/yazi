use std::{fmt::Display, str::FromStr, sync::atomic::{AtomicU64, Ordering}};

use serde::{Deserialize, Serialize};

#[derive(
	Clone, Copy, Debug, Default, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct Id(pub u64);

impl Id {
	#[inline]
	pub const fn get(&self) -> u64 { self.0 }

	pub fn unique() -> Self { Self(crate::timestamp_us()) }
}

impl Display for Id {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.0.fmt(f) }
}

impl FromStr for Id {
	type Err = <u64 as FromStr>::Err;

	fn from_str(s: &str) -> Result<Self, Self::Err> { s.parse().map(Self) }
}

impl From<u64> for Id {
	fn from(value: u64) -> Self { Self(value) }
}

impl From<usize> for Id {
	fn from(value: usize) -> Self { Self(value as u64) }
}

impl TryFrom<i64> for Id {
	type Error = <u64 as TryFrom<i64>>::Error;

	fn try_from(value: i64) -> Result<Self, Self::Error> { u64::try_from(value).map(Self) }
}

impl PartialEq<u64> for Id {
	fn eq(&self, other: &u64) -> bool { self.0 == *other }
}

// --- Ids
pub struct Ids {
	next: AtomicU64,
}

impl Ids {
	#[inline]
	pub const fn new() -> Self { Self { next: AtomicU64::new(1) } }

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
