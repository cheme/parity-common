
//! `metrics` macro



use srml_support_procedural_tools::syn_ext as ext;

use syn::Ident;
use syn::token::CustomKeyword;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
/*use srml_support_procedural_tools::{
  generate_crate_access,
  generate_hidden_includes,
};*/
 //  ([$($be:ident),*], $name:ident, $action:ident: $laz:expr, $level:ident, target: $target:expr, $($arg:tt)+)

/// Parsing usage only
#[derive(Parse, ToTokens, Debug)]
struct MetricsDefinition {
	pub hidden_crate: Option<SpecificHiddenCrate>,
	pub backends: ext::Brackets<ext::Punctuated<Ident, Token![,]>>,
  pub sep1: Token![,],
  pub name: Ident,
  pub sep2: Token![,],
  pub action: MetricsAction,
  pub sep3: Option<Token![,]>,
  pub logs: MetricsLogOption,
}
	
#[derive(Parse, ToTokens, Debug)]
struct MetricsAction {
  pub name: Ident,
  pub params: ext::Parens<ext::Punctuated<syn::Expr, Token![,]>>, 
}

#[derive(Parse, ToTokens, Debug)]
enum MetricsLogOption {
  MetricsLog(MetricsLog),
  None,
}
 
#[derive(Parse, ToTokens, Debug)]
struct MetricsLog {
  pub level: Ident, // can switch to enum if needed
  pub sep1: Token![,],
	pub target_keyword: ext::CustomToken<TargetKeyword>,
	pub target: syn::Expr,
  pub sep2: Option<Token![,]>,
  pub params: ext::Punctuated<syn::Expr, Token![,]>, 
}
 
#[derive(Parse, ToTokens, Debug)]
struct SpecificHiddenCrate {
	pub keyword: ext::CustomToken<SpecificHiddenCrate>,
	pub ident: ext::Parens<Ident>,
}

custom_keyword!(TargetKeyword, "target", "target as keyword");
custom_keyword_impl!(SpecificHiddenCrate, "from_crate", "hiddencrate as keyword");

pub fn generate_crate_access(def_crate: &str) -> TokenStream2 {
	if ::std::env::var("CARGO_PKG_NAME").unwrap().replace("-","_") == def_crate {
		quote!( crate )
	} else {
	  let ident = syn::Ident::new(def_crate, proc_macro2::Span::call_site());
		quote!( #ident )
	}.into()
}

pub fn metrics_impl(input: TokenStream) -> TokenStream {
	let def = parse_macro_input!(input as MetricsDefinition);
  let MetricsDefinition {
    hidden_crate,
    backends,
    logs,
    action,
    name,
    ..
  } = def;
	let hidden_crate_name = hidden_crate.map(|rc| rc.ident.content).map(|i| i.to_string())
		.unwrap_or_else(|| "metrics-backend".to_string());
/*	let scrate_decl = generate_hidden_includes(
		&"mbackend",
		"metrics-backends",
		"metrics_backends",
	);*/
  let scrate_decl = quote!();
	let scrate = generate_crate_access(&hidden_crate_name);
  let log = if let MetricsLogOption::MetricsLog(l) = logs {
    let MetricsLog {
      level,
      target,
      params,
      ..
    } = l;
    quote!{
      {
        use #scrate::log::log;
        #scrate::log::#level!(target: #target, #params);
      }
    }
  } else { quote!() };

  let calls = backends.content.inner.into_iter().fold(TokenStream2::new(), |mut st, be|{
    let action_name = &action.name; 
    let params = &action.params; 
    let call = quote!{
      let __ds = #scrate::#be::get_metrics_states().derived_state.#name.#action_name#params;
    };
    st.extend(call);
    st
  });
	
  let result = quote! {
    #scrate_decl
    #calls
    #log
  };
  result.into()
}
