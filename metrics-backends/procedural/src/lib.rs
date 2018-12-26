
//! Proc macro of `metrics-backends` crate.



#![recursion_limit="256"]

extern crate proc_macro;
extern crate proc_macro2;

#[macro_use]
extern crate syn;

#[macro_use]
extern crate quote;

#[macro_use]
extern crate srml_support_procedural_tools;
#[macro_use]
extern crate synstructure;

use proc_macro::TokenStream;

mod metrics;

mod modules;

/**
 * Main macro for declaring metrics usage.
 *
 * TODO syntax examples
 *  ([$($be:ident),*], $name:ident, $action:ident: $laz:expr, $level:ident, target: $target:expr, $($arg:tt)+)
 */
#[proc_macro]
pub fn metrics(input: TokenStream) -> TokenStream {
	metrics::metrics_impl(input)
}

#[proc_macro_attribute]
pub fn metrics_modules(attr: TokenStream, input: TokenStream) -> TokenStream {
	let metas = parse_macro_input!(attr as syn::AttributeArgs);
  modules::modules_impl(metas, input)
}
