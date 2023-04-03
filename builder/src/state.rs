use proc_macro2::Ident;

mod gen;

#[derive(Debug)]
pub(crate) struct State {
    pub(crate) name: Ident,
    pub(crate) methods: Vec<Method>,
}

#[derive(Debug)]
pub(crate) struct Method {
    pub(crate) name: Ident,
    pub(crate) ty: syn::Type,
    pub(crate) mode: Mode,
}

#[derive(Debug)]
pub(crate) enum Mode {
    Single,
    Optional,
    Sub(Option<String>),
}

impl State {
    pub(crate) fn new(name: Ident) -> State {
        let methods = Vec::new();

        State { name, methods }
    }
}
