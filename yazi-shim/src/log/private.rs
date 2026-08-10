use std::fmt;

pub use tracing_core::{Metadata, callsite::{Callsite, DefaultCallsite}, field::{FieldSet, Value}, identify_callsite, metadata::{Kind, Level}};

#[inline]
pub fn enabled(callsite: &'static DefaultCallsite) -> bool {
	let interest = callsite.interest();
	!interest.is_never()
		&& (interest.is_always()
			|| tracing_core::dispatcher::get_default(|dispatch| dispatch.enabled(callsite.metadata())))
}

#[inline]
pub fn emit(callsite: &'static DefaultCallsite, message: fmt::Arguments<'_>) {
	let metadata = callsite.metadata();
	let field = metadata.fields().field("message").unwrap();
	let values = [(&field, Some(&message as &dyn Value))];
	tracing_core::Event::dispatch(metadata, &metadata.fields().value_set(&values));
}
