use proc_macro2::Span;

#[derive(Debug)]
pub(crate) enum Error {
    IdentNeverSpecified,
    NeedNamedParams,
    PhaseMismatch,
    ParseError(syn::parse::Error),
}

impl Into<proc_macro::TokenStream> for Error {
    fn into(self) -> proc_macro::TokenStream {
        let parse_error = match self {
            Error::IdentNeverSpecified => {
                panic!()
            }

            Error::PhaseMismatch => syn::parse::Error::new(Span::call_site(), "Phase mismatch"),
            Error::NeedNamedParams => {
                syn::parse::Error::new(Span::call_site(), "Need named fields to generate setters")
            }
            Error::ParseError(e) => e,
        };

        parse_error.to_compile_error().into()
    }
}
