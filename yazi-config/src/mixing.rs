use std::{mem, sync::Arc};

pub(crate) trait Mixable {
	fn filter(&self) -> bool { true }

	fn any_file(&self) -> bool { false }

	fn any_dir(&self) -> bool { false }
}

impl<T: Mixable> Mixable for Arc<T> {
	fn filter(&self) -> bool { (**self).filter() }

	fn any_file(&self) -> bool { (**self).any_file() }

	fn any_dir(&self) -> bool { (**self).any_dir() }
}

pub(crate) fn mix<E, A, B, C>(a: A, b: B, c: C) -> Vec<E>
where
	E: Mixable,
	A: IntoIterator,
	A::Item: Into<E> + Mixable,
	B: IntoIterator,
	B::Item: Into<E> + Mixable,
	C: IntoIterator,
	C::Item: Into<E> + Mixable,
{
	fn dedup<E, I>(it: I, any_file: &mut bool, any_dir: &mut bool) -> impl Iterator<Item = E>
	where
		I: Iterator,
		I::Item: Into<E> + Mixable,
	{
		it.filter(move |x| {
			if !x.filter() {
				false
			} else if x.any_file() && mem::replace(any_file, true) {
				false
			} else if x.any_dir() && mem::replace(any_dir, true) {
				false
			} else {
				true
			}
		})
		.map(Into::into)
	}

	let (a, b, c) = (a.into_iter(), b.into_iter(), c.into_iter());
	let mut mixed = Vec::with_capacity(a.size_hint().0 + b.size_hint().0 + c.size_hint().0);

	let (mut a_any_file, mut a_any_dir) = (false, false);
	mixed.extend(dedup(a, &mut a_any_file, &mut a_any_dir));
	let a_len = mixed.len();

	mixed.extend(dedup(b, &mut a_any_file, &mut a_any_dir));
	let b_len = mixed.len();

	let (mut c_any_file, mut c_any_dir) = (false, false);
	mixed.extend(dedup(c, &mut c_any_file, &mut c_any_dir));

	if c_any_file || c_any_dir {
		let mut i = 0;
		mixed.retain(|x| {
			let in_b = (a_len..b_len).contains(&i);
			i += 1;

			if in_b && c_any_file && x.any_file() {
				return false;
			} else if in_b && c_any_dir && x.any_dir() {
				return false;
			}
			true
		})
	}

	mixed
}
