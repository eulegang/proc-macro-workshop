use syn::{visit::Visit, Attribute, Ident, Meta, Type};

pub struct Field {
    pub name: Ident,
    pub ty: Type,
    pub constraint: Option<syn::Expr>,
}

#[derive(Default)]
pub struct Fields {
    fields: Vec<Field>,
}

impl<'ast> Visit<'ast> for Fields {
    fn visit_field(&mut self, i: &'ast syn::Field) {
        let name = i.ident.clone().unwrap();
        let ty = i.ty.clone();

        let constraint = i
            .attrs
            .iter()
            .find(|a| a.meta.path().is_ident("bits"))
            .and_then(|attr| {
                if let Attribute {
                    meta: Meta::NameValue(syn::MetaNameValue { value, .. }),
                    ..
                } = attr
                {
                    Some(value.clone())
                } else {
                    None
                }
            });

        self.fields.push(Field {
            name,
            ty,
            constraint,
        })
    }
}

impl AsRef<[Field]> for Fields {
    fn as_ref(&self) -> &[Field] {
        self.fields.as_ref()
    }
}

impl std::ops::Deref for Fields {
    type Target = [Field];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
