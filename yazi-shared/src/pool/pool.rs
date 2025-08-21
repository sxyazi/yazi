use std::marker::PhantomData;

use hashbrown::hash_map::RawEntryMut;

use crate::pool::{SYMBOLS, Symbol, SymbolPtr, compute_hash};

pub struct Pool<T: ?Sized> {
	_phantom: PhantomData<T>,
}

impl Pool<[u8]> {
	pub fn intern(value: &[u8]) -> Symbol<[u8]> {
		let hash = compute_hash(value);

		match SYMBOLS.lock().raw_entry_mut().from_key_hashed_nocheck(hash, value) {
			RawEntryMut::Occupied(mut oe) => {
				let (ptr, count) = oe.get_key_value_mut();

				*count += 1;
				Symbol::new(ptr.clone())
			}
			RawEntryMut::Vacant(ve) => {
				let boxed = value.to_vec().into_boxed_slice();
				let ptr = SymbolPtr::leaked(Box::leak(boxed));

				ve.insert_hashed_nocheck(hash, ptr.clone(), 1);
				Symbol::new(ptr)
			}
		}
	}
}

impl Pool<str> {
	pub fn intern(value: impl AsRef<str>) -> Symbol<str> {
		let symbol = Pool::<[u8]>::intern(value.as_ref().as_bytes());
		Symbol::new(symbol.into_ptr())
	}
}
