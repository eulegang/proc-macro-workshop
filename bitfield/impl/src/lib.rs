use quote::quote;
use syn::{parse_macro_input, visit::Visit, DeriveInput};

mod fields;
mod getter;
mod setter;
mod size;

mod bitfield_specifier;

#[proc_macro_attribute]
pub fn bitfield(
    _: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut fields = fields::Fields::default();
    fields.visit_derive_input(&input);

    let name = &input.ident;

    let size = size::byte_size(&fields);
    let check = size::size_check(name, &fields);
    let constraint_check = size::size_constraint(&fields);

    let getters = getter::getters(&fields);
    let setters = setter::setters(&fields);

    quote! {
        #check
        #constraint_check

        struct #name {
            data: [u8; #size],
        }

        impl #name {
            fn new() -> Self {
                let data = [0; #size];
                Self { data }
            }

            #getters
            #setters
        }
    }
    .into()
}

#[proc_macro_derive(BitfieldSpecifier)]
pub fn bitfield_specifier(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match bitfield_specifier::derive(input) {
        Ok(t) => t.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
