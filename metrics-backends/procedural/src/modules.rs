// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.



//! Initiate metrics modules from a template struct

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use syn::NestedMeta;

fn struct_name(module: String) -> syn::Ident {
	let mut c = module.chars();
	let s_name = match c.next() {
		None => String::new(),
		Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
	};

	syn::Ident::new(&s_name, proc_macro2::Span::call_site())
}

pub fn modules_impl(metas: Vec<NestedMeta>, input: TokenStream) -> TokenStream {
	let item = parse_macro_input!(input as syn::ItemStruct);
/*	let scrate_decl: TokenStream = generate_hidden_includes(
		&"mbackend",
		"metrics-backends-tests",
		"metrics_backends_tests",
	).into();*/
//	result.extend(scrate_decl);

	let (mut result, init_fn, flush_fn, _input) = metas.into_iter().fold((TokenStream::new(), TokenStream2::new(), TokenStream2::new(), item),
		|(mut s, mut init_fn, mut flush_fn, input), m| {
		match m {
			NestedMeta::Meta(syn::Meta::Word(m)) => {
				let m_init = m.clone();
				s.extend(module_impl(m, &input));
				init_fn.extend(quote!{
					#m_init::init_metrics_states(&conf)?;
				});
				flush_fn.extend(quote!{
					#m_init::flush_metrics_states()?;
				});
			},
			_ => {
				panic!("TODO return error for unexpected arg");
			},
		};
		(s, init_fn, flush_fn, input)
	});
	let init_fn: TokenStream = quote!{
		pub fn init(conf: &GlobalCommonDef) -> Result<(), Error> {
			#init_fn
			#flush_fn
			Ok(())
		}
		pub fn flush() -> Result<(), Error> {
			#flush_fn
			Ok(())
		}
	}.into();
	result.extend(init_fn);
	result
}


pub fn module_impl(meta: syn::Ident, input: &syn::ItemStruct) -> TokenStream {
	let syn::ItemStruct {
		ref fields,
		..
	} = input;

	let meta_struct = struct_name(meta.to_string());

	let scrate = quote!(metrics_backends);

	let derived_fields = fields.iter().fold(TokenStream2::new(), |mut s, field| {
		let ty = &field.ty;
		let name = field.ident.as_ref().expect("TODO return error for unnamed field");
		s.extend(quote! {
			pub #name: #scrate::#meta::#ty,
		});
		s
	});
	let init_fields = fields.iter().fold(TokenStream2::new(), |mut s, field| {
		let ty = &field.ty;
		let name = field.ident.as_ref().expect("TODO return error for unnamed field");
		let sname = name.to_string();
		s.extend(quote! {
			#name: <#scrate::#meta::#ty as #scrate::#ty>::init(#sname, &global_state)?,
		});
		s
	});

	let result = quote!{
		pub mod #meta {
			use #scrate::{
				GlobalCommonDef,
				Backend,
				Error,
			};
			use #scrate::#meta::#meta_struct;
			#[derive(Clone)]
			pub struct DerivedStates {
				#derived_fields
			}
			#[derive(Clone)]
			pub struct States {
				pub global_state: <#meta_struct as Backend>::GlobalStates,
				pub derived_state: DerivedStates,
			}

			fn init_derived_state(global_state: &<#meta_struct as Backend>::GlobalStates) -> Result<DerivedStates, Error> {
				Ok(DerivedStates {
					#init_fields
				})
			}

			#[cfg(feature = "std")]
			static STATE: #scrate::once_cell::sync::OnceCell<States> = 
				#scrate::once_cell::sync::OnceCell::INIT;

			#[cfg(feature = "std")]
			pub fn init_metrics_states(conf: &GlobalCommonDef) -> Result<States, Error> {
				let global_state = <#meta_struct as Backend>::init_states(conf)?;
				let derived_state = init_derived_state(&global_state)?;
				<#meta_struct as Backend>::start_metrics(&global_state, conf.clone())?;
				Ok(States {
					global_state,
					derived_state,
				})
			}

			#[cfg(feature = "std")]
			pub fn init_metrics_states_panic(conf: &GlobalCommonDef) -> &'static States {
		//		STATE.get_or_try_init(|| {
				STATE.get_or_init(|| {
					match init_metrics_states(conf) {
						Ok(st) => st,
						Err(e) => panic!("Failed to initialize metrics backend: {}", e),
					}
		//			Ok(st)
				})
			}

			#[cfg(feature = "std")]
			pub fn get_metrics_states() -> &'static States {
				let conf = &<#meta_struct as Backend>::DEFAULT_CONF;
				init_metrics_states_panic(conf)
			}

			impl Drop for States {
				fn drop(&mut self) {
          // ignore error
					let _r = <#meta_struct as Backend>::async_write(&get_metrics_states().global_state);
				}
			}

			#[cfg(feature = "std")]
			pub fn flush_metrics_states() -> Result<(), Error> {
				<#meta_struct as Backend>::async_write(&get_metrics_states().global_state)?;
				Ok(())
			}
		}
	};
	result.into()
}
