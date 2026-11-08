use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

fn struct_fields(input: &DeriveInput) -> syn::Result<&Fields> {
    match &input.data {
        Data::Struct(data) => Ok(&data.fields),
        _ => Err(syn::Error::new_spanned(
            &input.ident,
            "only structs can be derived as packets",
        )),
    }
}

pub fn encode(input: DeriveInput) -> syn::Result<TokenStream> {
    let fields = struct_fields(&input)?;
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let members = fields.members();

    Ok(quote! {
        impl #impl_generics ::layline_codec::NetEncode for #name #ty_generics #where_clause {
            fn encode(&self) -> ::layline_codec::bytes::Bytes {
                #[allow(unused_mut)]
                let mut bytes = ::layline_codec::bytes::BytesMut::new();
                #(bytes.extend(::layline_codec::NetEncode::encode(&self.#members));)*
                bytes.freeze()
            }
        }
    })
}

pub fn decode(input: DeriveInput) -> syn::Result<TokenStream> {
    let fields = struct_fields(&input)?;
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let members = fields.members();
    let types = fields.iter().map(|field| &field.ty);

    Ok(quote! {
        impl #impl_generics ::layline_codec::NetDecode for #name #ty_generics #where_clause {
            fn decode(bytes: &mut ::layline_codec::bytes::Bytes) -> Option<Self> {
                Some(Self {
                    #(#members: <#types as ::layline_codec::NetDecode>::decode(bytes)?,)*
                })
            }
        }
    })
}
