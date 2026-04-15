use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, parse_macro_input};

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

	let assignments = match data {
		Data::Struct(struct_) => match struct_.fields {
			Fields::Named(fields) => {
				let mut assignments = Vec::with_capacity(fields.named.len());

				for field in fields.named {
					let field_ident = &field.ident;
					let field_name = field_ident.as_ref().unwrap().to_string();

					assignments.push(quote! {
						if let Some(value) = table.remove(#field_name) {
							if !matches!(value.get_ref(), toml::de::DeValue::Table(_)) {
								_ = deserialize_spanned::<toml::Table>(value)?;
								return Err(serde::de::Error::custom(format!("expected top-level `{}` to be a TOML table", #field_name)));
							}

							let span = value.span();
							if let toml::de::DeValue::Table(table) = value.into_inner() {
								self.#field_ident = self.#field_ident.deserialize_over_with(toml::Spanned::new(span, table))?;
							}
						}
						self.#field_ident = self.#field_ident.deserialize_over_hook()?;
					});
				}

				assignments
			}
			_ => panic!("DeserializeOver1 only supports structs with named fields"),
		},
		_ => panic!("DeserializeOver1 only supports structs"),
	};

	quote! {
		impl #impl_generics yazi_shim::toml::DeserializeOverWith for #ident #ty_generics #where_clause {
			fn deserialize_over_with<'de>(mut self, table: toml::Spanned<toml::de::DeTable<'de>>) -> Result<Self, toml::de::Error> {
				use yazi_shim::toml::{DeserializeOverHook, DeserializeOverWith, deserialize_spanned};

				let mut table = table.into_inner();
				#(#assignments)*

				self.deserialize_over_hook()
			}
		}
	}
	.into()
}

#[proc_macro_derive(DeserializeOver2)]
pub fn deserialize_over2(input: TokenStream) -> TokenStream {
	let DeriveInput { ident, generics, data, .. } = parse_macro_input!(input as DeriveInput);
	let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

	let assignments = match data {
		Data::Struct(struct_) => match struct_.fields {
			Fields::Named(fields) => {
				let mut assignments = Vec::with_capacity(fields.named.len());

				for (index, field) in fields.named.into_iter().enumerate() {
					let field_ident = field.ident;
					let field_name = field_ident.as_ref().unwrap().to_string();
					let field_ty = field.ty;
					let field_attrs = field.attrs;
					let field_helper = format_ident!("__DeserializeOver2Field{index}");

					assignments.push(quote! {
						#[derive(serde::Deserialize)]
						struct #field_helper #impl_generics (
							#(#field_attrs)*
							#field_ty
						) #where_clause;

						if let Some(value) = table.remove(#field_name) {
							self.#field_ident = deserialize_spanned::<#field_helper #ty_generics>(value)?.0;
						}
					});
				}

				assignments
			}
			_ => panic!("DeserializeOver2 only supports structs with named fields"),
		},
		_ => panic!("DeserializeOver2 only supports structs"),
	};

	quote! {
		impl #impl_generics yazi_shim::toml::DeserializeOverWith for #ident #ty_generics #where_clause {
			fn deserialize_over_with<'de>(mut self, table: toml::Spanned<toml::de::DeTable<'de>>) -> Result<Self, toml::de::Error> {
				use yazi_shim::toml::deserialize_spanned;

				let mut table = table.into_inner();
				#(#assignments)*

				Ok(self)
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
