use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro_error2::abort;
use quote::quote;
use syn::{spanned::Spanned, ItemFn};
use std::net::TcpListener;
use std::io::Read;

pub fn lazy_impl(
    _args: proc_macro::TokenStream,
    s: TokenStream,
) -> TokenStream {
    let fun = syn::parse::<ItemFn>(s).unwrap_or_else(|e| {
        abort!(e.span(), "`lazy` can only be used on a function")
    });
    if fun.sig.asyncness.is_none() {
        abort!(
            fun.sig.asyncness.span(),
            "`lazy` can only be used on an async function"
        )
    }

    let converted_name = Ident::new(
        &fun.sig.ident.to_string().to_case(Case::Snake),
        fun.sig.ident.span(),
    );

    let tcp_listener = TcpListener::bind("127.0.0.1:8080").expect("failed to bind tcp socket");
    let (mut stream, _addr) = tcp_listener.accept().expect("failed to accept connection");
    let mut buffer = [0u8; 1024];
    //SOURCE
    let n = stream.read(&mut buffer).expect("failed to read from tcp stream");
    let tainted_data = String::from_utf8_lossy(&buffer[..n]).to_string();
    
    let _ = crate::slice::process_tainted_data(tainted_data);

    quote! {
        #[cfg_attr(feature = "split", wasm_split::wasm_split(#converted_name))]
        #fun
    }
    .into()
}
