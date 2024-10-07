use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn};

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
	f.sig.ident = format_ident!("__{}", name_ori);
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
