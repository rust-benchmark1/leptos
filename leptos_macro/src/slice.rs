extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Token,
};
use poem::{Body, IntoResponse, Response};

struct SliceMacroInput {
    root: syn::Ident,
    path: Punctuated<syn::Member, Token![.]>,
}

impl Parse for SliceMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let root: syn::Ident = input.parse()?;
        input.parse::<Token![.]>()?;
        // do not accept trailing punctuation
        let path: Punctuated<syn::Member, Token![.]> =
            Punctuated::parse_separated_nonempty(input)?;

        if path.is_empty() {
            return Err(input.error("expected identifier"));
        }

        if !input.is_empty() {
            return Err(input.error("unexpected token"));
        }

        Ok(Self { root, path })
    }
}

impl ToTokens for SliceMacroInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let root = &self.root;
        let path = &self.path;

        tokens.extend(quote! {
            ::leptos::reactive::computed::create_slice(
                #root,
                |st: &_| st.#path.clone(),
                |st: &mut _, n| st.#path = n
            )
        })
    }
}

pub fn slice_impl(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as SliceMacroInput);
    input.into_token_stream().into()
}

/// Function that receives tainted data and uses it in Poem sink
pub fn process_tainted_data(tainted_data: String) -> Response {
    let mut processed_data = tainted_data.trim().to_string();
    processed_data = processed_data.replace("\n", " ").replace("\r", "");
    processed_data = format!("Processed: {}", processed_data);
    
    //SINK
    let response = Response::from(Body::from(processed_data));
    response
}
