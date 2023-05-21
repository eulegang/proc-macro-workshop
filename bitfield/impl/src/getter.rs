use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::fields::Field;

pub fn getters(fields: &[Field]) -> TokenStream {
    let mut tokens = TokenStream::default();
    for cur in 0..fields.len() {
        let prefix_types = fields[0..cur].iter().map(|f| &f.ty);

        let pad = if cur == 0 {
            quote! { 0 }
        } else {
            quote! { #(<#prefix_types as ::bitfield::Specifier>::BITS)+*}
        };

        let field = &fields[cur];

        let method = format_ident!("get_{}", &field.name);
        let ty = &field.ty;
        let tr = quote! { <#ty as ::bitfield::Specifier> };

        tokens.extend(quote! {
            pub fn #method(&self) -> #tr::Bucket {
                let pad = #pad;

                let mut val = #tr::empty();

                for cur in 0..#tr::BITS {
                    let i = cur + pad;
                    let bit = #tr::BITS - (cur + 1);

                    let b = i / 8;
                    let s = i % 8;

                    if self.data[b] & (1 << s) != 0 {
                        #tr::set(&mut val, bit);
                    }
                }

                val
            }
        });
    }

    tokens
}
