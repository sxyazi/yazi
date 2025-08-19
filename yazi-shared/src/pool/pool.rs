use std::marker::PhantomData;

use crate::pool::{SYMBOLS, Symbol, SymbolPtr};

pub struct Pool<T: ?Sized> {
	_phantom: PhantomData<T>,
}

impl Pool<[u8]> {
	pub fn intern(value: &[u8]) -> Symbol<[u8]> {
		let mut lock = SYMBOLS.lock();

		if let Some((ptr, count)) = lock.get_key_value_mut(value) {
			*count += 1;
			return Symbol::new(ptr.clone());
		}

		let boxed = value.to_vec().into_boxed_slice();
		let ptr = SymbolPtr::leaked(Box::leak(boxed));

		lock.insert(ptr.clone(), 1);
		Symbol::new(ptr)
	}
}

impl Pool<str> {
	pub fn intern(value: impl AsRef<str>) -> Symbol<str> {
		let symbol = Pool::<[u8]>::intern(value.as_ref().as_bytes());
		Symbol::new(symbol.into_ptr())
	}
}
