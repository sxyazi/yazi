use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Data, DeriveInput, Fields, FnArg, ItemFn, ext::IdentExt, parse_macro_input};

#[proc_macro_attribute]
pub fn command(_: TokenStream, item: TokenStream) -> TokenStream {
	let mut f: ItemFn = syn::parse(item).unwrap();
	let mut ins = f.sig.inputs.clone();

	// Turn `opt: Opt` into `opt: impl Into<Opt>`
	ins[1] = {
		let FnArg::Typed(opt) = &f.sig.inputs[1] else {
			panic!("Cannot find the `opt` argument in the function signature.");
		};

		let opt_ty = &opt.ty;
		syn::parse2(quote! { opt: impl Into<#opt_ty> }).unwrap()
	};

	// Make the original function private and add a public wrapper
	assert!(matches!(f.vis, syn::Visibility::Public(_)));
	f.vis = syn::Visibility::Inherited;

	// Add `__` prefix to the original function name
	let name_ori = f.sig.ident;
	f.sig.ident = format_ident!("__{}", name_ori.unraw());
	let name_new = &f.sig.ident;

	// Collect the rest of the arguments
	let rest_args = ins.iter().skip(2).map(|arg| match arg {
		FnArg::Receiver(_) => unreachable!(),
		FnArg::Typed(t) => &t.pat,
	});

	quote! {
		#[inline]
		pub fn #name_ori(#ins) { self.#name_new(opt.into(), #(#rest_args),*); }
		#f
	}
	.into()
}

#[proc_macro_derive(DeserializeOver1)]
pub fn deserialize_over1(input: TokenStream) -> TokenStream {
	// Parse the input tokens into a syntax tree
	let input = parse_macro_input!(input as DeriveInput);

	// Get the name of the struct
	let name = &input.ident;
	let shadow_name = format_ident!("__{name}Shadow");

	// Process the struct fields
	let (shadow_fields, field_calls) = match &input.data {
		Data::Struct(struct_) => match &struct_.fields {
			Fields::Named(fields) => {
				let mut shadow_fields = Vec::with_capacity(fields.named.len());
				let mut field_calls = Vec::with_capacity(fields.named.len());

				for field in &fields.named {
					let name = &field.ident;
					let attrs: Vec<&Attribute> =
						field.attrs.iter().filter(|&a| a.path().is_ident("serde")).collect();

					shadow_fields.push(quote! {
							#(#attrs)*
							pub(crate) #name: Option<toml::Value>
					});
					field_calls.push(quote! {
						if let Some(value) = shadow.#name {
							self.#name = self.#name.deserialize_over(value).map_err(serde::de::Error::custom)?;
						}
					});
				}

				(shadow_fields, field_calls)
			}
			_ => panic!("DeserializeOver1 only supports structs with named fields"),
		},
		_ => panic!("DeserializeOver1 only supports structs"),
	};

	quote! {
		#[derive(serde::Deserialize)]
		pub(crate) struct #shadow_name {
			#(#shadow_fields),*
		}

		impl #name {
			#[inline]
			pub(crate) fn deserialize_over<'de, D>(self, deserializer: D) -> Result<Self, D::Error>
			where
				D: serde::Deserializer<'de>,
			{
				self.deserialize_over_with::<D>(Self::deserialize_shadow(deserializer)?)
			}

			#[inline]
			pub(crate) fn deserialize_shadow<'de, D>(deserializer: D) -> Result<#shadow_name, D::Error>
			where
				D: serde::Deserializer<'de>,
			{
				#shadow_name::deserialize(deserializer)
			}

			#[inline]
			pub(crate) fn deserialize_over_with<'de, D>(mut self, shadow: #shadow_name) -> Result<Self, D::Error>
			where
				D: serde::Deserializer<'de>,
			{
				#(#field_calls)*
				Ok(self)
			}
		}
	}
	.into()
}

#[proc_macro_derive(DeserializeOver2)]
pub fn deserialize_over2(input: TokenStream) -> TokenStream {
	// Parse the input tokens into a syntax tree
	let input = parse_macro_input!(input as DeriveInput);

	// Get the name of the struct
	let name = &input.ident;
	let shadow_name = format_ident!("__{name}Shadow");

	// Process the struct fields
	let (shadow_fields, field_assignments) = match &input.data {
		Data::Struct(struct_) => match &struct_.fields {
			Fields::Named(fields) => {
				let mut shadow_fields = Vec::with_capacity(fields.named.len());
				let mut field_assignments = Vec::with_capacity(fields.named.len());

				for field in &fields.named {
					let (ty, name) = (&field.ty, &field.ident);
					shadow_fields.push(quote! {
						pub(crate) #name: Option<#ty>
					});
					field_assignments.push(quote! {
						if let Some(value) = shadow.#name {
							self.#name = value;
						}
					});
				}

				(shadow_fields, field_assignments)
			}
			_ => panic!("DeserializeOver2 only supports structs with named fields"),
		},
		_ => panic!("DeserializeOver2 only supports structs"),
	};

	quote! {
		#[derive(serde::Deserialize)]
		pub(crate) struct #shadow_name {
			#(#shadow_fields),*
		}

		impl #name {
			#[inline]
			pub(crate) fn deserialize_over<'de, D>(mut self, deserializer: D) -> Result<Self, D::Error>
			where
				D: serde::Deserializer<'de>
			{
				Ok(self.deserialize_over_with(Self::deserialize_shadow(deserializer)?))
			}

			#[inline]
			pub(crate) fn deserialize_shadow<'de, D>(deserializer: D) -> Result<#shadow_name, D::Error>
			where
				D: serde::Deserializer<'de>
			{
				#shadow_name::deserialize(deserializer)
			}

			#[inline]
			pub(crate) fn deserialize_over_with(mut self, shadow: #shadow_name) -> Self {
				#(#field_assignments)*
				self
			}
		}
	}
	.into()
}
