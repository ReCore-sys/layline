use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

pub fn expand(item: ItemFn) -> syn::Result<TokenStream> {
    if item.sig.asyncness.is_some() {
        return Err(syn::Error::new_spanned(
            item.sig.asyncness,
            "packet handlers cannot be async",
        ));
    }
    if item.sig.inputs.len() != 3 {
        return Err(syn::Error::new_spanned(
            &item.sig.inputs,
            "packet handlers take a world, a source and a packet",
        ));
    }
    Ok(quote!(#item))
}
