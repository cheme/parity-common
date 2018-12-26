
//! `metrics` macro



use srml_support_procedural_tools::syn_ext as ext;

use syn::Ident;
use syn::token::CustomKeyword;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use srml_support_procedural_tools::generate_crate_access;
 //  ([$($be:ident),*], $name:ident, $action:ident: $laz:expr, $level:ident, target: $target:expr, $($arg:tt)+)


/// Parsing usage only
#[derive(Parse, ToTokens, Debug)]
struct MetricsDefinition {
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
 
custom_keyword!(TargetKeyword, "target", "target as keyword");


pub fn metrics_impl(input: TokenStream) -> TokenStream {
	let def = parse_macro_input!(input as MetricsDefinition);
  let MetricsDefinition {
    backends,
    logs,
    action,
    name,
    ..
  } = def;
	
	let scrate = generate_crate_access(&"metrics_backends", "metrics-backends");
  let log = if let MetricsLogOption::MetricsLog(l) = logs {
    let MetricsLog {
      level,
      target,
      params,
      ..
    } = l;
    quote!{
      #scrate::log::#level!(target: #target, #params);
    }
  } else { quote!() };

  let calls = backends.content.inner.into_iter().fold(TokenStream2::new(), |mut st, be|{
    let action_name = &action.name; 
    let params = &action.params; 
    let call = quote!{
      #scrate::#be::#action_name::#name#params;
    };
    st.extend(call);
    st
  });
	
  let result = quote! {

    #calls
    #log
  };
  result.into()
}


