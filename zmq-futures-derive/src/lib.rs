#![feature(proc_macro, proc_macro_lib)]

extern crate proc_macro;
use proc_macro::TokenStream;

extern crate syn;

#[macro_use]
extern crate quote;

#[proc_macro_derive(Builder)]
pub fn builder(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let ast = syn::parse_derive_input(&source).unwrap();
    let expanded = expand_builder(&ast);
    quote!(#expanded).to_string().parse().unwrap()
}

fn expand_builder(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;

    let socket_type = syn::Ident::from(format!("zmq::{}", format!("{}", name).to_uppercase()));

    let builder_name = syn::Ident::from(format!("{}Builder", name));
    let bind_builder_name = syn::Ident::from(format!("{}BindBuilder", name));
    let connect_builder_name = syn::Ident::from(format!("{}ConnectBuilder", name));

    quote! {
        pub enum #builder_name {
            Sock(Rc<zmq::Socket>),
            Fail(zmq::Error),
        }

        impl #builder_name {
            pub fn new() -> Self {
                let ctx = zmq::Context::new();

                match ctx.socket(#socket_type) {
                    Ok(sock) => #builder_name::Sock(Rc::new(sock)),
                    Err(e) => #builder_name::Fail(e),
                }
            }

            pub fn bind(self, addr: &str) -> #bind_builder_name {
                match self {
                    #builder_name::Sock(sock) => {
                        #bind_builder_name::Sock(sock).bind(addr)
                    }
                    #builder_name::Fail(e) => #bind_builder_name::Fail(e)
                }
            }

            pub fn connect(self, addr: &str) -> #connect_builder_name {
                match self {
                    #builder_name::Sock(sock) => {
                        #connect_builder_name::Sock(sock).connect(addr)
                    }
                    #builder_name::Fail(e) => #connect_builder_name::Fail(e)
                }
            }
        }

        pub enum #bind_builder_name {
            Sock(Rc<zmq::Socket>),
            Fail(zmq::Error),
        }

        impl #bind_builder_name {
            pub fn bind(self, addr: &str) -> Self {
                match self {
                    #bind_builder_name::Sock(sock) => {
                        match sock.bind(addr) {
                            Ok(_) => #bind_builder_name::Sock(sock),
                            Err(e) => #bind_builder_name::Fail(e),
                        }
                    }
                    #bind_builder_name::Fail(e) => #bind_builder_name::Fail(e)
                }
            }

            pub fn build(self) -> zmq::Result<#name> {
                match self {
                    #bind_builder_name::Sock(sock) => Ok(#name { sock }),
                    #bind_builder_name::Fail(e) => Err(e),
                }
            }
        }

        pub enum #connect_builder_name {
            Sock(Rc<zmq::Socket>),
            Fail(zmq::Error),
        }

        impl #connect_builder_name {
            pub fn connect(self, addr: &str) -> Self {
                match self {
                    #connect_builder_name::Sock(sock) => {
                        match sock.connect(addr) {
                            Ok(_) => #connect_builder_name::Sock(sock),
                            Err(e) => #connect_builder_name::Fail(e),
                        }
                    }
                    #connect_builder_name::Fail(e) => #connect_builder_name::Fail(e)
                }
            }

            pub fn build(self) -> zmq::Result<#name> {
                match self {
                    #connect_builder_name::Sock(sock) => Ok(#name { sock }),
                    #connect_builder_name::Fail(e) => Err(e),
                }
            }
        }
    }
}

#[proc_macro_derive(SinkSocket)]
pub fn sink_socket(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let ast = syn::parse_derive_input(&source).unwrap();
    let expanded = expand_sink_socket(&ast);
    quote!(#expanded).to_string().parse().unwrap()
}

fn expand_sink_socket(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;

    quote! {
        impl<E: From<zmq::Error>> ::SinkSocket<E> for #name {}
    }
}

#[proc_macro_derive(StreamSocket)]
pub fn stream_socket(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let ast = syn::parse_derive_input(&source).unwrap();
    let expanded = expand_stream_socket(&ast);
    quote!(#expanded).to_string().parse().unwrap()
}

fn expand_stream_socket(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    quote! {
        impl #impl_generics ::StreamSocket for #name #ty_generics #where_clause {}
    }
}

#[proc_macro_derive(ZmqSocket)]
pub fn zmq_socket(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let ast = syn::parse_derive_input(&source).unwrap();
    let expanded = expand_socket_field(&ast);
    quote!(#expanded).to_string().parse().unwrap()
}

fn expand_socket_field(ast: &syn::DeriveInput) -> quote::Tokens {
    let s = match ast.body {
        syn::Body::Struct(ref data) => data,
        syn::Body::Enum(_) => panic!("Cannot derive Socket for enums"),
    };

    let f = match *s {
        syn::VariantData::Struct(ref fields) => fields,
        syn::VariantData::Tuple(ref fields) => fields,
        syn::VariantData::Unit => panic!("Cannot derive Socket for Unit structs"),
    };

    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut count = 0;
    for field in f {
        if let Some(item) = get_last_path_item(&field.ty) {
            if let Some(_) = expand_path_item(item) {
                if let Some(ref field_ident) = field.ident {
                    return quote! {
                        impl #impl_generics ::ZmqSocket for #name #ty_generics #where_clause {
                            fn socket(&self) -> Rc<zmq::Socket> {
                                Rc::clone(&self.#field_ident)
                            }
                        }
                    };
                } else {
                    return quote! {
                        impl #impl_generics ::ZmqSocket for #name #ty_generics #where_clause {
                            fn socket(&self) -> Rc<zmq::Socket> {
                                Rc::clone(&self.#count)
                            }
                        }
                    };
                }
            }
        }

        count += 1;
    }

    panic!("No socket type found in struct");
}

fn get_last_path_item(path: &syn::Ty) -> Option<&syn::PathSegment> {
    if let syn::Ty::Path(_, ref path) = *path {
        if path.segments.len() > 0 {
            return path.segments.get(path.segments.len() - 1);
        }
    }

    None
}

fn expand_path_item(item: &syn::PathSegment) -> Option<&syn::PathSegment> {
    if item.ident == syn::Ident::from("Rc") {
        if let syn::PathParameters::AngleBracketed(ref data) = item.parameters {
            for ty in &data.types {
                if let Some(ref inner) = get_last_path_item(&ty) {
                    if inner.ident == syn::Ident::from("Socket") {
                        return Some(item);
                    }
                }
            }
        }
    }

    return None;
}
