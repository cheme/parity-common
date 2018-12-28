
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

/// compied from syn crate 0.15.23 (non public)
struct NamedDecl<'a>(&'a syn::FnDecl, &'a syn::Ident);

/// compied from syn crate 0.15.23 (non public)
impl<'a> quote::ToTokens for NamedDecl<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.fn_token.to_tokens(tokens);
        self.1.to_tokens(tokens);
        self.0.generics.to_tokens(tokens);
        self.0.paren_token.surround(tokens, |tokens| {
            self.0.inputs.to_tokens(tokens);
            if self.0.variadic.is_some() && !self.0.inputs.empty_or_trailing() {
                <Token![,]>::default().to_tokens(tokens);
            }
            self.0.variadic.to_tokens(tokens);
        });
        self.0.output.to_tokens(tokens);
        self.0.generics.where_clause.to_tokens(tokens);
    }
}


fn is_outer(attr: &&syn::Attribute) -> bool {
  use syn::AttrStyle;
      match attr.style {
                AttrStyle::Outer => true,
                _ => false,
     }
}
fn is_inner(attr: &&syn::Attribute) -> bool {
  use syn::AttrStyle;
      match attr.style {
                AttrStyle::Inner(..) => true,
                _ => false,
     }
}
 
#[proc_macro_attribute]
pub fn timer_enclose(attr: TokenStream, input: TokenStream) -> TokenStream {


  use quote::{ToTokens, TokenStreamExt};

	let metas = parse_macro_input!(attr as syn::AttributeArgs);
  let (m_name, macro_backends) = if metas.len() == 0 {
      panic!("no metrics name");
  } else if metas.len() == 1 {
    (if let Some(syn::NestedMeta::Meta(syn::Meta::Word(ref m))) = metas.get(0) {
      quote!(#m)
    } else {
      panic!("unexpected metrics name");
    },
 
    quote!(timer_enclose_backends))
  } else {
    (if let Some(syn::NestedMeta::Meta(syn::Meta::Word(ref m))) = metas.get(0) {
      quote!(#m)
    } else {
      panic!("unexpected metrics name");
    },
 
    if let Some(syn::NestedMeta::Meta(syn::Meta::Word(ref m))) = metas.get(1) {
      quote!(#m)
    } else {
      panic!("unexpected macro name");
    })
  };
  let syn::ItemFn {
    attrs,
    vis,
    constness,
    unsafety,
    asyncness,
    abi,
    ident,
    decl,
    block,
  } = parse_macro_input!(input as syn::ItemFn);

  let start = quote!{
    #macro_backends!(#m_name, start());
    let mut r = move ||
  };
  let end = quote!{
    ;
    let r = r();
    #macro_backends!(#m_name, suspend());
    r
  };
  let mut tokens = proc_macro2::TokenStream::new();
  // logic from ToTokens impl of ItemFn in syn crate 0.15.23
  tokens.append_all(attrs.iter().filter(is_outer));
  vis.to_tokens(&mut tokens);
  constness.to_tokens(&mut tokens);
  unsafety.to_tokens(&mut tokens);
  asyncness.to_tokens(&mut tokens);
  abi.to_tokens(&mut tokens);
  NamedDecl(&decl, &ident).to_tokens(&mut tokens);
  block.brace_token.surround(&mut tokens, |mut tokens| {
    tokens.append_all(start.into_iter());
    tokens.append_all(attrs.iter().filter(is_inner));
    block.brace_token.surround(&mut tokens, |tokens| {
       tokens.append_all(&block.stmts);
    });
    tokens.append_all(end.into_iter());
  });
 
  tokens.into()

}
