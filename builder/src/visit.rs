use proc_macro2::Ident;
use syn::{
    visit::{visit_derive_input, visit_field, Visit},
    Attribute, DeriveInput, Type,
};

use crate::{
    error::Error,
    state::{Method, Mode, State},
    Builder,
};

impl<'ast> Visit<'ast> for Builder {
    fn visit_derive_input(&mut self, i: &'ast DeriveInput) {
        match self.state {
            Err(Error::IdentNeverSpecified) => self.state = Ok(State::new(i.ident.clone())),
            _ => (),
        }

        visit_derive_input(self, i);
    }

    fn visit_field(&mut self, i: &'ast syn::Field) {
        let Some(ident) = &i.ident else {
            self.state = Err(Error::NeedNamedParams);
            return;
        };

        match &mut self.state {
            Ok(state) => {
                if let Some(ty) = extract_vec(&i.ty) {
                    let name = match extract_each_attr(&i.attrs) {
                        Ok(name) => name,
                        Err(e) => {
                            self.state = Err(Error::ParseError(e));

                            return;
                        }
                    };

                    if let Some(name) = name {
                        state.methods.push(Method {
                            name: ident.clone(),
                            ty: ty.clone(),
                            mode: Mode::Sub(Some(name)),
                        });
                    } else {
                        state.methods.push(Method {
                            name: ident.clone(),
                            ty: ty.clone(),
                            mode: Mode::Sub(None),
                        });
                    }
                } else if let Some(ty) = extract_option(&i.ty) {
                    state.methods.push(Method {
                        name: ident.clone(),
                        ty: ty.clone(),
                        mode: Mode::Optional,
                    });
                } else {
                    state.methods.push(Method {
                        name: ident.clone(),
                        ty: i.ty.clone(),
                        mode: Mode::Single,
                    });
                }
            }

            _ => match self.state {
                Ok(_) => self.state = Err(Error::PhaseMismatch),
                _ => (),
            },
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
