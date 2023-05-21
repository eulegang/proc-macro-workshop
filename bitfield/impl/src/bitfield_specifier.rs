use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{DeriveInput, Error};

pub fn derive(input: DeriveInput) -> syn::Result<TokenStream> {
    let e = match input.data {
        syn::Data::Struct(_) => {
            return Err(Error::new(
                input.ident.span(),
                "BitfieldSpecifier does not support structs",
            ))
        }
        syn::Data::Union(_) => {
            return Err(Error::new(
                input.ident.span(),
                "BitfieldSpecifier does not support unions",
            ))
        }
        syn::Data::Enum(e) => e,
    };

    let bits = {
        let len = e.variants.len();

        if len.is_power_of_two() {
            (usize::BITS - len.leading_zeros()) as usize - 1
        } else {
            return Err(Error::new(
                Span::call_site(),
                "BitfieldSpecifier expected a number of variants which is a power of 2",
            ));
        }
    };

    let subty = match bits {
        1..=8 => quote! { <B8 as ::bitfield::Specifier> },
        9..=16 => quote! { <B8 as ::bitfield::Specifier> },
        17..=32 => quote! { <B8 as ::bitfield::Specifier> },
        33..=64 => quote! { <B8 as ::bitfield::Specifier> },

        _ => panic!("wtf"),
    };

    let ty_name = &input.ident;

    let match_expr = {
        let names = e.variants.iter().map(|v| {
            let ident = &v.ident;
            let constant = format_ident!("{}", ident.to_string().to_uppercase());

            quote! {
                const #constant: #subty::Bucket = #ty_name::#ident as #subty::Bucket;
            }
        });

        let arms = e.variants.iter().map(|v| {
            let ident = &v.ident;
            let constant = format_ident!("{}", ident.to_string().to_uppercase());

            quote! {
                #constant => Self::#ident,
            }
        });

        quote! {
            {
                #(#names)*

                match val {
                    #(#arms)*
                    _ => todo!("unspecified value"),
                }
            }
        }
    };

    let mut checks = TokenStream::default();

    for var in &e.variants {
        let var_name = &var.ident;
        let msg = format!("{var_name} is out of range for type {ty_name}");
        let cap = e.variants.len();

        checks.extend(quote! {
            const _: () = assert!(#cap as #subty::Bucket > #ty_name::#var_name as #subty::Bucket  , #msg);
        });
    }

    Ok(quote! {
        #checks

        impl ::std::clone::Clone for #ty_name {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl ::std::marker::Copy for #ty_name { }

        impl Specifier for #ty_name {
            type Bucket = Self;
            const BITS: usize = #bits;

            fn is_set(bucket: &Self, bit: usize) -> bool {
                #subty::is_set(&(*bucket as #subty::Bucket), bit)
            }

            fn set(bucket: &mut Self, bit: usize) {
                let mut val = #subty::empty();
                val = *bucket as #subty::Bucket;
                #subty::set(&mut val, bit);
                *bucket = #match_expr;

                //#subty::set(&mut (*bucket as #subty::Bucket), bit);
            }

            fn empty() -> Self {
                let val = 0;

                #match_expr

            }
        }
    })
}
