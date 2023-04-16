use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input, visit::Visit, visit_mut::VisitMut, Attribute, ExprMatch, Item, ItemEnum,
    ItemFn, Pat,
};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let item = parse_macro_input!(input as Item);

    let mut checker = Checker::default();
    checker.visit_item(&item);

    if let Some(err) = checker.err {
        let tokens: proc_macro2::TokenStream = err.into_compile_error().into();
        quote! { #item #tokens }.into()
    } else {
        quote! { #item }.into()
    }
}

#[proc_macro_attribute]
pub fn check(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_fn = parse_macro_input!(input as ItemFn);

    let mut check = FnChecker::default();
    check.visit_item_fn_mut(&mut item_fn);

    if let Some(err) = check.err {
        let tokens: proc_macro2::TokenStream = err.into_compile_error().into();
        quote! { #item_fn #tokens }.into()
    } else {
        quote! { #item_fn }.into()
    }
}

#[derive(Default)]
struct Checker {
    err: Option<syn::Error>,
}

impl<'a> Visit<'a> for Checker {
    fn visit_item(&mut self, item: &Item) {
        match item {
            Item::Enum(_) => (),

            _ => {
                self.err = Some(syn::Error::new(
                    Span::call_site(),
                    "expected enum or match expression",
                ));
            }
        };

        syn::visit::visit_item(self, item)
    }

    fn visit_item_enum(&mut self, e: &ItemEnum) {
        let vec: Vec<_> = e.variants.iter().collect();

        for (i, var) in vec.iter().enumerate() {
            let name = var.ident.to_string();

            for prior in &vec[..i] {
                let n = prior.ident.to_string();

                if n > name {
                    if self.err.is_none() {
                        self.err = Some(syn::Error::new_spanned(
                            &var.ident,
                            format!("{} should sort before {}", name, n),
                        ));
                    }
                }
            }
        }
    }
}

#[derive(Default)]
struct FnChecker {
    err: Option<syn::Error>,
}

impl VisitMut for FnChecker {
    fn visit_expr_match_mut(&mut self, expr: &mut ExprMatch) {
        if needs_sorted(&mut expr.attrs) {
            let mut names = Vec::with_capacity(expr.arms.len());

            for arm in &expr.arms {
                if let Some(name) = pattern(&arm.pat) {
                    names.push(name)
                } else {
                    if self.err.is_none() {
                        dbg!(arm);
                        self.err = Some(syn::Error::new_spanned(
                            &arm.pat,
                            format!("unsupported by #[sorted]"),
                        ));
                        return;
                    }
                }
            }

            for (i, arm) in expr.arms.iter().enumerate() {
                let name = &names[i];

                for (j, _) in expr.arms[..i].iter().enumerate() {
                    let n = &names[j];

                    if n > name {
                        if self.err.is_none() {
                            if let Pat::TupleStruct(s) = &arm.pat {
                                self.err = Some(syn::Error::new_spanned(
                                    &s.path,
                                    format!("{} should sort before {}", name, n),
                                ));
                            }

                            /*
                            self.err = Some(syn::Error::new(
                                *span,
                                format!("{} should sort before {}", name, n),
                            ));
                            */
                        }
                    }
                }
            }
        }

        syn::visit_mut::visit_expr_match_mut(self, expr);
    }
}

fn needs_sorted(attrs: &mut Vec<Attribute>) -> bool {
    let mut mfd = Vec::with_capacity(attrs.len());

    for (i, attr) in attrs.iter().enumerate() {
        if let syn::Meta::Path(p) = &attr.meta {
            if p.is_ident("sorted") {
                mfd.push(i);
            }
        }
    }

    let need = !mfd.is_empty();

    for i in mfd.iter().rev() {
        attrs.remove(*i);
    }

    need
}

fn pattern(pat: &Pat) -> Option<String> {
    let mut name = String::new();

    match pat {
        Pat::TupleStruct(s) => {
            let mut first = true;
            for p in s.path.segments.iter() {
                if !first {
                    name.push_str("::");
                } else {
                    first = false;
                }
                name.push_str(&p.ident.to_string());
            }

            Some(name)
        }

        Pat::Ident(i) => {
            name.push_str(&i.ident.to_string());

            Some(name)
        }

        Pat::Wild(_) => {
            name.push('_');

            Some(name)
        }
        _ => None,
    }
}
