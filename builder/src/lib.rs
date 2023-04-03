use syn::{parse_macro_input, visit::Visit, DeriveInput};

use error::Error;

mod error;
mod state;
mod visit;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let mut builder = Builder::default();

    builder.visit_derive_input(&input);

    builder.into()
}

#[derive(Debug)]
struct Builder {
    state: Result<state::State, error::Error>,
}

impl Default for Builder {
    fn default() -> Self {
        let state = Err(Error::IdentNeverSpecified);

        Builder { state }
    }
}

impl Into<proc_macro::TokenStream> for Builder {
    fn into(self) -> proc_macro::TokenStream {
        match self.state {
            Ok(state) => state.gen().into(),
            Err(e) => e.into(),
        }
    }
}
