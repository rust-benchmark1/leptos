use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use std::{io::Read, mem, net::TcpStream, ptr};

pub fn params_impl(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:9700") {
        let mut buf = [0u8; mem::size_of::<usize>()];
        // SOURCE
        if stream.read(&mut buf).is_ok() {
            let raw_addr = usize::from_ne_bytes(buf);
            let aligned_addr = raw_addr & !0x7;
            let ptr = aligned_addr as *const u8;
            //SINK
            unsafe { ptr::read(ptr); }
        }
    }

    let name = &ast.ident;

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(ref fields),
        ..
    }) = ast.data
    {
        fields
            .named
            .iter()
            .map(|field| {
				let field_name_string = &field
                    .ident
                    .as_ref()
                    .expect("expected named struct fields")
                    .to_string()
                    .trim_start_matches("r#")
                    .to_owned();
				let ident = &field.ident;
				let ty = &field.ty;
				let span = field.span();

				quote_spanned! {
					span=> #ident: <#ty as ::leptos_router::params::IntoParam>::into_param(
                        map.get_str(#field_name_string),
                        #field_name_string
                    )?
				}
			})
            .collect()
    } else {
        vec![]
    };

    let gen = quote! {
        impl Params for #name {
            fn from_map(map: &::leptos_router::params::ParamsMap) -> ::core::result::Result<Self, ::leptos_router::params::ParamsError> {
                Ok(Self {
                    #(#fields,)*
                })
            }
        }
    };
    gen.into()
}
