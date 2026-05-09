use proc_macro2::Span;
use syn::{Attribute, Data, Field, Fields, GenericParam, Ident, Lifetime, LifetimeParam, Meta, punctuated::Punctuated, token::Comma};

pub(super) fn named_fields(data: Data) -> Punctuated<Field, Comma> {
	match data {
		Data::Struct(s) => match s.fields {
			Fields::Named(f) => f.named,
			_ => panic!("expected named fields"),
		},
		_ => panic!("expected struct"),
	}
}

pub(super) fn ident_name(ident: &Ident) -> String {
	let name = ident.to_string();
	match name.strip_prefix("r#") {
		Some(s) => s.to_owned(),
		None => name,
	}
}

pub(super) fn generics_with_de(generics: &syn::Generics) -> syn::Generics {
	let mut g = generics.clone();
	g.params.insert(
		0,
		GenericParam::Lifetime(LifetimeParam::new(Lifetime::new("'__de", Span::call_site()))),
	);
	g
}

pub(super) fn has_serde_attr(attrs: &[Attribute], name: &str) -> bool {
	attrs.iter().any(|a| {
		a.path().is_ident("serde")
			&& a
				.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
				.is_ok_and(|n| n.iter().any(|m| m.path().is_ident(name)))
	})
}
