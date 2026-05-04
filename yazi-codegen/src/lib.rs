use proc_macro::TokenStream;
use quote::quote;
mod helper;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

use crate::helper::{generics_with_de, has_serde_attr, ident_name, named_fields};

#[proc_macro_derive(DeserializeOver)]
pub fn deserialize_over(input: TokenStream) -> TokenStream {
	let DeriveInput { ident, generics, .. } = parse_macro_input!(input as DeriveInput);
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	quote! {
		impl #impl_generics yazi_shim::toml::DeserializeOverHook for #ident #ty_generics #where_clause {}
	}
	.into()
}

#[proc_macro_derive(DeserializeOver1)]
pub fn deserialize_over1(input: TokenStream) -> TokenStream {
	let DeriveInput { ident, generics, data, .. } = parse_macro_input!(input as DeriveInput);
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let visitor_generics = generics_with_de(&generics);
	let (impl_visitor_generics, ..) = visitor_generics.split_for_impl();

	let (flatten_fields, normal_fields): (Vec<_>, Vec<_>) =
		named_fields(data).into_iter().partition(|f| has_serde_attr(&f.attrs, "flatten"));

	let field_hooks: Vec<_> = flatten_fields
		.iter()
		.chain(&normal_fields)
		.map(|f| {
			let ident = f.ident.as_ref().unwrap();
			quote! { #ident: deserialized.#ident.deserialize_over_hook().map_err(Error::custom)? }
		})
		.collect();

	let normal_arms = normal_fields.into_iter().map(|f| {
		let ident = f.ident.unwrap();
		let name = ident_name(&ident);
		quote! { #name => self.0.#ident = map.next_value_seed(DeserializeOverSeed(self.0.#ident))? }
	});

	let flatten_arm = match flatten_fields.into_iter().next() {
		Some(f) => {
			let ident = f.ident.unwrap();
			quote! { _ => self.0.#ident = self.0.#ident.deserialize_over_with(single_map_entry(key, &mut map))? }
		}
		None => quote! { _ => _ = map.next_value::<IgnoredAny>()? },
	};

	quote! {
		impl #impl_generics yazi_shim::toml::DeserializeOverWith for #ident #ty_generics #where_clause {
			fn deserialize_over_with<'__de, __D: serde::Deserializer<'__de>>(self, de: __D) -> Result<Self, __D::Error> {
				use serde::de::{Error, IgnoredAny, MapAccess, Visitor};
				use yazi_shared::KebabCasedString;
				use yazi_shim::{serde::single_map_entry, toml::{DeserializeOverHook, DeserializeOverSeed, DeserializeOverWith}};

				struct V #impl_generics (#ident #ty_generics) #where_clause;

				impl #impl_visitor_generics Visitor<'__de> for V #ty_generics #where_clause {
					type Value = #ident #ty_generics;

					fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
						f.write_str("a map")
					}

					fn visit_map<__M: MapAccess<'__de>>(mut self, mut map: __M) -> Result<Self::Value, __M::Error> {
						while let Some(key) = map.next_key::<KebabCasedString>()? {
							match key.as_ref() {
								#(#normal_arms,)*
								#flatten_arm
							}
						}
						Ok(self.0)
					}
				}

				let deserialized = de.deserialize_map(V(self))?;
				Ok(Self { #(#field_hooks,)* })
			}
		}
	}
	.into()
}

#[proc_macro_derive(DeserializeOver2)]
pub fn deserialize_over2(input: TokenStream) -> TokenStream {
	let DeriveInput { ident, generics, data, .. } = parse_macro_input!(input as DeriveInput);
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let visitor_generics = generics_with_de(&generics);
	let (impl_visitor_generics, ..) = visitor_generics.split_for_impl();

	let mut normal_arms = vec![];
	let mut flatten_arm = quote! { _ => _ = map.next_value::<IgnoredAny>()? };
	for field in named_fields(data) {
		let (field_ident, field_ty) = (field.ident, field.ty);
		let field_name = ident_name(field_ident.as_ref().unwrap());

		if has_serde_attr(&field.attrs, "skip") {
			continue;
		}

		if has_serde_attr(&field.attrs, "flatten") {
			flatten_arm = quote! { _ => self.0.#field_ident = self.0.#field_ident.deserialize_over_with(single_map_entry(key, &mut map))? };
			continue;
		}

		let serde_attrs: Vec<_> = field.attrs.iter().filter(|a| a.path().is_ident("serde")).collect();
		if serde_attrs.is_empty() {
			normal_arms.push(quote! { #field_name => self.0.#field_ident = map.next_value()? });
		} else {
			normal_arms.push(quote! {
				#field_name => {
					#[derive(serde::Deserialize)]
					struct H #impl_generics(#(#serde_attrs)* #field_ty,) #where_clause;
					self.0.#field_ident = map.next_value::<H #ty_generics>()?.0;
				}
			});
		}
	}

	quote! {
		impl #impl_generics yazi_shim::toml::DeserializeOverWith for #ident #ty_generics #where_clause {
			fn deserialize_over_with<'__de, __D: serde::Deserializer<'__de>>(self, de: __D) -> Result<Self, __D::Error> {
				use serde::de::{Error, IgnoredAny, MapAccess, Visitor};
				use std::borrow::Cow;
				use yazi_shim::{serde::single_map_entry, toml::DeserializeOverWith};

				struct V #impl_generics (#ident #ty_generics) #where_clause;

				impl #impl_visitor_generics Visitor<'__de> for V #ty_generics #where_clause {
					type Value = #ident #ty_generics;

					fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
						f.write_str("a map")
					}

					fn visit_map<__M: MapAccess<'__de>>(mut self, mut map: __M) -> Result<Self::Value, __M::Error> {
						while let Some(key) = map.next_key::<Cow<str>>()? {
							match key.as_ref() {
								#(#normal_arms,)*
								#flatten_arm
							}
						}

						Ok(self.0)
					}
				}

				de.deserialize_map(V(self))
			}
		}
	}
	.into()
}

#[proc_macro_derive(Overlay)]
pub fn overlay(input: TokenStream) -> TokenStream {
	let DeriveInput { ident, generics, data, .. } = parse_macro_input!(input as DeriveInput);
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let stmts: Vec<_> = match data {
		Data::Struct(s) => match s.fields {
			Fields::Named(fields) => fields
				.named
				.into_iter()
				.map(|f| {
					let field_ident = f.ident;
					quote! { self.#field_ident.overlay(new.#field_ident); }
				})
				.collect(),
			Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
				vec![quote! { self.0.overlay(new.0); }]
			}
			_ => panic!("expected named fields or a single-field tuple struct"),
		},
		_ => panic!("expected struct"),
	};

	quote! {
		impl #impl_generics yazi_shim::serde::Overlay for #ident #ty_generics #where_clause {
			fn overlay(&self, new: Self) {
				use yazi_shim::serde::Overlay;

				#(#stmts)*
			}
		}
	}
	.into()
}

#[proc_macro_derive(FromLuaOwned)]
pub fn from_lua(input: TokenStream) -> TokenStream {
	let DeriveInput { ident, generics, .. } = parse_macro_input!(input as DeriveInput);

	let ident_str = ident.to_string();
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	quote! {
		impl #impl_generics ::mlua::FromLua for #ident #ty_generics #where_clause {
			#[inline]
			fn from_lua(value: ::mlua::Value, _: &::mlua::Lua) -> ::mlua::Result<Self> {
				match value {
					::mlua::Value::UserData(ud) => ud.take::<Self>(),
					_ => Err(::mlua::Error::FromLuaConversionError {
							from: value.type_name(),
							to: #ident_str.to_owned(),
							message: None,
					}),
				}
			}
		}
	}
	.into()
}
