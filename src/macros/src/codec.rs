use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{Data, DeriveInput, Fields, GenericArgument, Ident, Type};

fn struct_fields(input: &DeriveInput) -> syn::Result<&Fields> {
    match &input.data {
        Data::Struct(data) => Ok(&data.fields),
        _ => Err(syn::Error::new_spanned(
            &input.ident,
            "only structs can be derived as packets",
        )),
    }
}

fn field_conditions(fields: &Fields) -> syn::Result<Vec<Option<Ident>>> {
    let conditions = fields
        .iter()
        .map(|field| {
            let mut condition = None;
            for attr in &field.attrs {
                if attr.path().is_ident("conditional") {
                    if condition.is_some() {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "a field can only have one `conditional` attribute",
                        ));
                    }
                    condition = Some(attr.parse_args::<Ident>()?);
                }
            }
            Ok(condition)
        })
        .collect::<syn::Result<Vec<_>>>()?;

    if conditions.iter().any(Option::is_some) && !matches!(fields, Fields::Named(_)) {
        return Err(syn::Error::new_spanned(
            fields,
            "conditional fields require a struct with named fields",
        ));
    }

    if conditions.iter().any(Option::is_some) {
        let named_fields = match fields {
            Fields::Named(fields) => &fields.named,
            _ => unreachable!(),
        };
        let positions: HashMap<_, _> = named_fields
            .iter()
            .enumerate()
            .map(|(index, field)| (field.ident.as_ref().unwrap().to_string(), index))
            .collect();

        for (index, condition) in conditions.iter().enumerate() {
            let Some(condition) = condition else {
                continue;
            };
            let field = &named_fields[index];
            let Some(&condition_index) = positions.get(&condition.to_string()) else {
                return Err(syn::Error::new_spanned(
                    condition,
                    "conditional field selector must name an earlier `bool` field",
                ));
            };
            if condition_index >= index
                || !matches!(
                    &named_fields[condition_index].ty,
                    Type::Path(ty) if ty.qself.is_none() && ty.path.is_ident("bool")
                )
            {
                return Err(syn::Error::new_spanned(
                    condition,
                    "conditional field selector must name an earlier `bool` field",
                ));
            }

            if option_inner_type(&field.ty).is_none() {
                return Err(syn::Error::new_spanned(
                    &field.ty,
                    "conditional fields must have type `Option<T>`",
                ));
            }
        }
    }

    Ok(conditions)
}

fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(ty) = ty else {
        return None;
    };
    if ty.qself.is_some() || ty.path.segments.last()?.ident != "Option" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &ty.path.segments.last()?.arguments else {
        return None;
    };
    match args.args.first()? {
        GenericArgument::Type(inner) if args.args.len() == 1 => Some(inner),
        _ => None,
    }
}

pub fn encode(input: DeriveInput) -> syn::Result<TokenStream> {
    let fields = struct_fields(&input)?;
    let conditions = field_conditions(fields)?;
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let writes = fields
        .members()
        .zip(conditions)
        .map(|(member, condition)| {
            if let Some(condition) = condition {
                quote! {
                    if self.#condition {
                        let value = self.#member.as_ref().expect(
                            "conditional field must be Some when its condition is true"
                        );
                        bytes.extend(::layline_codec::NetEncode::encode(value));
                    }
                }
            } else {
                quote! {
                    bytes.extend(::layline_codec::NetEncode::encode(&self.#member));
                }
            }
        });

    Ok(quote! {
        impl #impl_generics ::layline_codec::NetEncode for #name #ty_generics #where_clause {
            fn encode(&self) -> ::layline_codec::bytes::Bytes {
                #[allow(unused_mut)]
                let mut bytes = ::layline_codec::bytes::BytesMut::new();
                #(#writes)*
                bytes.freeze()
            }
        }
    })
}

pub fn decode(input: DeriveInput) -> syn::Result<TokenStream> {
    let fields = struct_fields(&input)?;
    let conditions = field_conditions(fields)?;
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    if conditions.iter().any(Option::is_some) {
        let named_fields = match fields {
            Fields::Named(fields) => &fields.named,
            _ => unreachable!(),
        };
        let locals: Vec<_> = (0..named_fields.len())
            .map(|index| format_ident!("__layline_field_{index}"))
            .collect();
        let decodes = named_fields
            .iter()
            .zip(&locals)
            .zip(&conditions)
            .map(|((field, local), condition)| {
                let ty = &field.ty;
                if let Some(condition) = condition {
                    let condition_index = named_fields
                        .iter()
                        .position(|candidate| candidate.ident.as_ref() == Some(condition))
                        .unwrap();
                    let condition_local = &locals[condition_index];
                    let inner = option_inner_type(ty).unwrap();
                    quote! {
                        let #local = if #condition_local {
                            Some(<#inner as ::layline_codec::NetDecode>::decode(bytes)?)
                        } else {
                            None
                        };
                    }
                } else {
                    quote! {
                        let #local = <#ty as ::layline_codec::NetDecode>::decode(bytes)?;
                    }
                }
            });
        let names = named_fields.iter().map(|field| field.ident.as_ref().unwrap());

        return Ok(quote! {
            impl #impl_generics ::layline_codec::NetDecode for #name #ty_generics #where_clause {
                fn decode(bytes: &mut ::layline_codec::bytes::Bytes) -> Option<Self> {
                    #(#decodes)*
                    Some(Self {
                        #(#names: #locals,)*
                    })
                }
            }
        });
    }

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
