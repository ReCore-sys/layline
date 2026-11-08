mod codec;
mod handler;
mod packet;

use proc_macro::TokenStream;
use syn::{DeriveInput, ItemFn, parse_macro_input};

/// Encodes each field in declaration order using its `NetEncode` implementation.
#[proc_macro_derive(NetEncode)]
pub fn derive_net_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    codec::encode(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Decodes each field in declaration order using its `NetDecode` implementation.
#[proc_macro_derive(NetDecode)]
pub fn derive_net_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    codec::decode(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Implements `Packet` using the matching entry of the `packets.json` next to the
/// calling crate's `Cargo.toml`.
///
/// ```ignore
/// #[derive(NetDecode)]
/// #[packet(name = "Login", inbound)]
/// pub struct Login {
///     pub username: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn packet(args: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as DeriveInput);
    packet::expand(args.into(), item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Marks a packet handler so the `build.rs` of the crate registers it.
///
/// Handlers have the signature `fn(&mut World, &Source, &mut Packet)`.
#[proc_macro_attribute]
pub fn handler(_args: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemFn);
    handler::expand(item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
