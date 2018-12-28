//! Initiate metrics modules from a template struct

use srml_support_procedural_tools::syn_ext as ext;

use syn::Ident;
use syn::token::CustomKeyword;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use srml_support_procedural_tools::{
  generate_crate_access,
  generate_hidden_includes,
};

use syn::NestedMeta;

pub fn modules_impl(metas: Vec<NestedMeta>, input: TokenStream) -> TokenStream {
	let item = parse_macro_input!(input as syn::ItemStruct);
/*  let scrate_decl: TokenStream = generate_hidden_includes(
		&"mbackend",
		"metrics-backends-tests",
		"metrics_backends_tests",
	).into();*/
//  result.extend(scrate_decl);

  let (mut result, init_fn, input) = metas.into_iter().fold((TokenStream::new(),TokenStream2::new(), item),
    |(mut s, mut init_fn, input), m| {
    match m {
      NestedMeta::Meta(syn::Meta::Word(m)) => {
        let m_init = m.clone();
        s.extend(module_impl(m, &input));
        init_fn.extend(quote!{
          #m_init::init_metrics_states(&conf);
        });
      },
      _ => {
        panic!("TODO return error for unexpected arg");
      },
    };
    (s, init_fn, input)
  });
  let init_fn: TokenStream = quote!{
    pub fn init(conf: &GlobalCommonDef) {
      #init_fn
    }
  }.into();
  result.extend(init_fn);
  result
}


pub fn module_impl(meta: syn::Ident, input: &syn::ItemStruct) -> TokenStream {
  let syn::ItemStruct {
		ref ident,
		ref fields,
		..
	} = input;

/*  // only named fields allowed
  let fields = if let &syn::Fields::Named(syn::FieldsNamed{ref named, ..}) = fields {
    named
  } else { 
    panic!("TODO return error for unexpected arg");
  };*/

		let scrate_decl = quote!();

//	let scrate = generate_crate_access(&"mbackend", "metrics-backends-tests");
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
      #name: #scrate::#meta::#ty::init(#sname, &global_state),
    });
    s
  });

  let result = quote!{
    pub mod #meta {
      use #scrate::{
        GlobalCommonDef,
      };
      use #scrate::#meta::{
        GlobalStates,
        DEFAULT_CONF,
        init_states,
        start_metrics,
        async_write,
      }; 
      #[derive(Clone)]
      pub struct DerivedStates {
        #derived_fields
      }
      #[derive(Clone)]
      pub struct States {
        pub global_state: GlobalStates,
        pub derived_state: DerivedStates,
      }
      fn init_derived_state(global_state: &GlobalStates) -> DerivedStates {
        DerivedStates {
          #init_fields
        }
      }

      #[cfg(feature = "std")]
      static STATE: #scrate::once_cell::sync::OnceCell<States> = 
        #scrate::once_cell::sync::OnceCell::INIT;

      #[cfg(feature = "std")]
      pub fn init_metrics_states(conf: &GlobalCommonDef) -> &'static States {
    //    STATE.get_or_try_init(|| {
        STATE.get_or_init(|| {
          let global_state = init_states(conf);
          let derived_state = init_derived_state(&global_state);
          start_metrics(&global_state, conf.clone())
            .expect("Fail on metrics states initialization");
          let st = States {
            global_state,
            derived_state,
          };

          st
    //      Ok(st)
        })
      }

      #[cfg(feature = "std")]
      pub fn get_metrics_states() -> &'static States {
        let conf = &DEFAULT_CONF;
        init_metrics_states(conf)
      }

      #[cfg(feature = "std")]
      impl Drop for States {
        fn drop(&mut self) {
          // TODO if right mode (no need to gate that behind macro)
          async_write(&get_metrics_states().global_state)
        }
      }
    }
  };
  result.into()
}
