use crate::pool::{Pool, Symbol};

pub trait InternStr {
	fn intern(&self) -> Symbol<str>;
}

impl<T: AsRef<str>> InternStr for T {
	fn intern(&self) -> Symbol<str> { Pool::<str>::intern(self) }
}
