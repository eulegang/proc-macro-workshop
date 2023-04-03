use anyhow::anyhow;
use proc_macro2::Ident;
use proc_macro2::Span;
use quote::format_ident;
use quote::quote;
use syn::visit::{visit_derive_input, visit_field, Visit};
use syn::Attribute;
use syn::Type;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut builder = Builder::default();

    builder.visit_derive_input(&input);

    let out: proc_macro::TokenStream = builder.into();
    out
}

#[derive(Default, Debug)]
enum Builder {
    #[default]
    Blank,
    Ident(Ident, Vec<Method>),
    Error(anyhow::Error),
    PError(syn::parse::Error),
}

#[derive(Debug)]
struct Method {
    name: Ident,
    ty: syn::Type,
    mode: Mode,
}

#[derive(Debug)]
enum Mode {
    Single,
    Optional,
    Sub(Option<String>),
}

impl Into<proc_macro::TokenStream> for Builder {
    fn into(self) -> proc_macro::TokenStream {
        match self {
            Builder::Error(err) => syn::parse::Error::new(Span::call_site(), format!("{err}"))
                .into_compile_error()
                .into(),

            Builder::PError(err) => err.into_compile_error().into(),
            Builder::Blank => quote! {}.into(),

            Builder::Ident(ident, methods) => {
                let builder_ident = format_ident!("{}Builder", ident);

                let fields = methods.iter().map(|method| {
                    let name = &method.name;
                    let ty = &method.ty;
                    match &method.mode {
                        Mode::Single | Mode::Optional => {
                            quote! { #name: ::std::option::Option<#ty> }
                        }
                        Mode::Sub(None) => quote! { #name: ::std::option::Option<Vec<#ty>> },
                        Mode::Sub(_) => quote! { #name: ::std::vec::Vec<#ty> },
                    }
                });

                let setters = methods.iter().map(|method| {
                    let name = &method.name;
                    let ty = &method.ty;

                    match &method.mode {
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
                });

                let inits = methods.iter().map(|method| {
                    let name = format_ident!("{}", &method.name);
                    match method.mode {
                        Mode::Single | Mode::Optional | Mode::Sub(None) => {
                            quote! { let #name = ::std::option::Option::None; }
                        }
                        Mode::Sub(_) => quote! { let #name = ::std::vec::Vec::new(); },
                    }
                });

                let names = methods
                    .iter()
                    .map(|method| {
                        let name = &method.name;
                        quote! { #name }
                    })
                    .collect::<Vec<_>>();

                let unwraps = methods.iter().map(|method| {
                    let name = &method.name;

                    match method.mode {
                        Mode::Single | Mode::Sub(None) => {
                            quote! { let #name = self.#name.as_ref()?.clone(); }
                        }
                        Mode::Optional => quote! { let #name = self.#name.clone(); },
                        Mode::Sub(Some(_)) => quote! { let #name = self.#name.clone(); },
                    }
                });

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
                .into()
            }
        }
    }
}

impl Builder {
    fn error(&mut self, e: anyhow::Error) {
        match self {
            Builder::Error(_) => (),
            Builder::PError(_) => (),
            _ => *self = Builder::Error(e),
        }
    }
}

impl<'ast> Visit<'ast> for Builder {
    fn visit_derive_input(&mut self, i: &'ast DeriveInput) {
        match self {
            Builder::Blank => *self = Builder::Ident(i.ident.clone(), vec![]),

            _ => self.error(anyhow!("Phase mismatch needed blank found {:?}", self)),
        }

        visit_derive_input(self, i);
    }

    fn visit_field(&mut self, i: &'ast syn::Field) {
        let Some(ident) = &i.ident else {
            self.error(anyhow!("a field needs an identifier to be considered for a builder"));

            return;
        };

        match self {
            Builder::Ident(_, methods) => {
                if let Some(ty) = extract_vec(&i.ty) {
                    let name = match extract_each_attr(&i.attrs) {
                        Ok(name) => name,
                        Err(e) => {
                            *self = Builder::PError(e);

                            return;
                        }
                    };

                    if let Some(name) = name {
                        methods.push(Method {
                            name: ident.clone(),
                            ty: ty.clone(),
                            mode: Mode::Sub(Some(name)),
                        });
                    } else {
                        methods.push(Method {
                            name: ident.clone(),
                            ty: ty.clone(),
                            mode: Mode::Sub(None),
                        });
                    }
                } else if let Some(ty) = extract_option(&i.ty) {
                    methods.push(Method {
                        name: ident.clone(),
                        ty: ty.clone(),
                        mode: Mode::Optional,
                    });
                } else {
                    methods.push(Method {
                        name: ident.clone(),
                        ty: i.ty.clone(),
                        mode: Mode::Single,
                    });
                }
            }

            _ => self.error(anyhow!("Phase mismatch needed ident found {:?}", self)),
        }

        visit_field(self, i)
    }
}

fn extract_option(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else { return None };
    let Some(segment) = path.path.segments.first() else { return None };
    if segment.ident != "Option" {
        return None;
    };

    let syn::PathArguments::AngleBracketed(ref args) = segment.arguments else { return None };
    if args.args.len() != 1 {
        return None;
    };
    let arg: &syn::GenericArgument = args.args.first()?;
    let syn::GenericArgument::Type(ref ty) = arg else { return None };

    Some(ty)
}

fn extract_vec(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else { return None };
    let Some(segment) = path.path.segments.first() else { return None };
    if segment.ident != "Vec" {
        return None;
    };

    let syn::PathArguments::AngleBracketed(ref args) = segment.arguments else { return None };
    if args.args.len() != 1 {
        return None;
    };
    let arg: &syn::GenericArgument = args.args.first()?;
    let syn::GenericArgument::Type(ref ty) = arg else { return None };

    Some(ty)
}

fn extract_each_attr(attrs: &[Attribute]) -> Result<Option<String>, syn::parse::Error> {
    for attr in attrs {
        let syn::Meta::List(ref list) = attr.meta else { continue; };
        if !list.path.is_ident("builder") {
            continue;
        }

        if let Ok(assign) = list.parse_args::<Assign>() {
            if assign.ident == "each" {
                let syn::Lit::Str(s) = assign.value else { continue };
                return Ok(Some(s.value()));
            } else {
                return Err(syn::parse::Error::new_spanned(
                    list,
                    "expected `builder(each = \"...\")`",
                ));
            }
        }
    }

    Ok(None)
}

struct Assign {
    ident: Ident,
    value: syn::Lit,
}

impl syn::parse::Parse for Assign {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        let _: syn::Token![=] = input.parse()?;
        let value = input.parse()?;

        Ok(Assign { ident, value })
    }
}
