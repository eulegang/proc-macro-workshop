use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::{Method, Mode, State};

impl State {
    pub(crate) fn gen(&self) -> TokenStream {
        let builder_ident = format_ident!("{}Builder", self.name);
        let ident = format_ident!("{}", self.name);

        let fields = self.methods.iter().map(Method::field);
        let setters = self.methods.iter().map(Method::setter);
        let inits = self.methods.iter().map(Method::inits);
        let names = self.methods.iter().map(Method::name).collect::<Vec<_>>();
        let unwraps = self.methods.iter().map(Method::unwrap);

        quote! {
            pub struct #builder_ident {
                #(#fields),*
            }

            impl #builder_ident {
                #(#setters)*

                pub fn build(&self) -> ::std::option::Option<#ident> {
                    #(#unwraps)*

                    Some(#ident { #(#names),* })
                }
            }

            impl #ident {
                pub fn builder() -> #builder_ident {
                    #(#inits);*

                    #builder_ident { #(#names),* }
                }
            }
        }
    }
}

impl Method {
    fn field(&self) -> TokenStream {
        let name = &self.name;
        let ty = &self.ty;
        match &self.mode {
            Mode::Single | Mode::Optional => {
                quote! { #name: ::std::option::Option<#ty> }
            }
            Mode::Sub(None) => quote! { #name: ::std::option::Option<Vec<#ty>> },
            Mode::Sub(_) => quote! { #name: ::std::vec::Vec<#ty> },
        }
    }

    fn setter(&self) -> TokenStream {
        let name = &self.name;
        let ty = &self.ty;

        match &self.mode {
            Mode::Single | Mode::Optional => {
                quote! {
                    pub fn #name(&mut self, #name: #ty) -> &mut Self {
                        self.#name = ::std::option::Option::Some(#name);

                        self
                    }
                }
            }

            Mode::Sub(None) => {
                quote! {
                    pub fn #name(&mut self, #name: ::std::vec::Vec<#ty>) -> &mut Self {
                        self.#name = ::std::option::Option::Some(#name);

                        self
                    }
                }
            }

            Mode::Sub(Some(each)) => {
                let each = format_ident!("{}", each);
                if name == &each {
                    quote! {
                        pub fn #each(&mut self, #name: #ty) -> &mut Self {
                            self.#name.push(#name);

                            self
                        }
                    }
                } else {
                    quote! {
                        pub fn #name(&mut self, #name: ::std::vec::Vec<#ty>) -> &mut Self {
                            self.#name.extend(#name);

                            self
                        }

                        pub fn #each(&mut self, #name: #ty) -> &mut Self {
                            self.#name.push(#name);

                            self
                        }
                    }
                }
            }
        }
    }

    fn inits(&self) -> TokenStream {
        let name = format_ident!("{}", &self.name);
        match self.mode {
            Mode::Single | Mode::Optional | Mode::Sub(None) => {
                quote! { let #name = ::std::option::Option::None; }
            }
            Mode::Sub(_) => quote! { let #name = ::std::vec::Vec::new(); },
        }
    }

    fn name(&self) -> TokenStream {
        let name = &self.name;
        quote! { #name }
    }

    fn unwrap(&self) -> TokenStream {
        let name = &self.name;

        match self.mode {
            Mode::Single | Mode::Sub(None) => {
                quote! { let #name = self.#name.as_ref()?.clone(); }
            }
            Mode::Optional => quote! { let #name = self.#name.clone(); },
            Mode::Sub(Some(_)) => quote! { let #name = self.#name.clone(); },
        }
    }
}
