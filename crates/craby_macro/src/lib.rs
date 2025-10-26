use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, ImplItem, ItemImpl};

#[proc_macro_attribute]
pub fn craby_module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);

    let has_new = input
        .items
        .iter()
        .any(|item| matches!(item, ImplItem::Fn(method) if method.sig.ident == "new"));

    let has_id = input
        .items
        .iter()
        .any(|item| matches!(item, ImplItem::Fn(method) if method.sig.ident == "id"));

    if !has_new {
        let new_method: ImplItem = parse_quote! {
            fn new(ctx: Context) -> Self {
                Self { ctx }
            }
        };
        input.items.push(new_method);
    }

    if !has_id {
        let id_method: ImplItem = parse_quote! {
            fn id(&self) -> usize {
                self.ctx.id
            }
        };
        input.items.push(id_method);
    }

    TokenStream::from(quote! { #input })
}
