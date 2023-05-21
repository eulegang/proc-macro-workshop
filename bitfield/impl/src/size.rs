use crate::fields::Field;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Ident;

pub fn byte_size(fields: &[Field]) -> TokenStream {
    if fields.is_empty() {
        quote! { 0 }
    } else {
        let s = fields.iter().map(|f| &f.ty);
        quote! {
            (#(<#s as ::bitfield::Specifier>::BITS)+*) / 8
        }
    }
}

pub fn size_check(name: &Ident, fields: &[Field]) -> TokenStream {
    if fields.is_empty() {
        quote! {}
    } else {
        let s = fields.iter().map(|f| &f.ty);
        let message = format!("{name} does not fill all bytes");

        quote! {
            const _: () = assert!(( #(<#s as ::bitfield::Specifier>::BITS)+* ) % 8 == 0, #message);
        }
    }
}

pub fn size_constraint(fields: &[Field]) -> TokenStream {
    let mut tokens = TokenStream::default();

    for field in fields {
        if let Some(expr) = &field.constraint {
            let ty = &field.ty;

            let ty_repr = {
                let mut t = TokenStream::default();
                ty.to_tokens(&mut t);
                t.to_string()
            };
            let expr_repr = {
                let mut t = TokenStream::default();
                expr.to_tokens(&mut t);
                t.to_string()
            };

            let msg = format!("expected {ty_repr} to be of size {expr_repr}");

            tokens.extend(quote! {
                const _: () = assert!(#expr == <#ty as ::bitfield::Specifier>::BITS, #msg);

            });
        }
    }

    tokens
}
