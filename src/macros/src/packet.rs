use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::{env, fs};
use syn::parse::Parser;
use syn::{DeriveInput, Ident, LitStr};

struct Args {
    name: LitStr,
    direction: Ident,
}

impl Args {
    fn parse(tokens: TokenStream) -> syn::Result<Self> {
        let mut name = None;
        let mut direction = None;
        syn::meta::parser(|meta| {
            if meta.path.is_ident("name") {
                name = Some(meta.value()?.parse::<LitStr>()?);
            } else if meta.path.is_ident("inbound") {
                direction = Some(format_ident!("Inbound"));
            } else if meta.path.is_ident("outbound") {
                direction = Some(format_ident!("Outbound"));
            } else {
                return Err(meta.error("expected `name = \"..\"`, `inbound` or `outbound`"));
            }
            Ok(())
        })
        .parse2(tokens)?;

        let missing = |what| syn::Error::new(Span::call_site(), format!("missing {what}"));
        Ok(Self {
            name: name.ok_or_else(|| missing("`name = \"..\"`"))?,
            direction: direction.ok_or_else(|| missing("`inbound` or `outbound`"))?,
        })
    }
}

struct Definition {
    id: i16,
    reliability: Ident,
    channel: u8,
}

impl Definition {
    fn load(name: &LitStr, path: &Path) -> syn::Result<Self> {
        let error = |message: String| syn::Error::new(name.span(), message);
        let contents = fs::read_to_string(path)
            .map_err(|e| error(format!("failed to read {}: {e}", path.display())))?;
        let packets: Value = serde_json::from_str(&contents)
            .map_err(|e| error(format!("failed to parse {}: {e}", path.display())))?;
        let packet = packets
            .get(name.value())
            .ok_or_else(|| error(format!("`{}` is not defined in packets.json", name.value())))?;

        let number = |key: &str| {
            packet
                .get(key)
                .and_then(Value::as_i64)
                .ok_or_else(|| error(format!("`{}` has no numeric `{key}`", name.value())))
        };
        let reliability = packet
            .get("reliability")
            .and_then(Value::as_str)
            .ok_or_else(|| error(format!("`{}` has no `reliability`", name.value())))?;

        Ok(Self {
            id: i16::try_from(number("id")?).map_err(|_| error("packet id exceeds i16".into()))?,
            reliability: Ident::new(reliability, name.span()),
            channel: u8::try_from(number("channel")?)
                .map_err(|_| error("ordering channel exceeds u8".into()))?,
        })
    }
}

pub fn expand(args: TokenStream, item: DeriveInput) -> syn::Result<TokenStream> {
    let Args { name, direction } = Args::parse(args)?;
    let manifest = env::var("CARGO_MANIFEST_DIR")
        .map_err(|e| syn::Error::new(name.span(), format!("CARGO_MANIFEST_DIR: {e}")))?;
    let path = PathBuf::from(manifest).join("packets.json");
    let Definition {
        id,
        reliability,
        channel,
    } = Definition::load(&name, &path)?;

    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
    let path = path.to_string_lossy();

    Ok(quote! {
        #item

        impl #impl_generics ::layline_codec::Packet for #ident #ty_generics #where_clause {
            const PACKET_ID: i16 = #id;
            const DIRECTION: ::layline_codec::PacketDirection =
                ::layline_codec::PacketDirection::#direction;
            const PACKET_NAME: &'static str = #name;
            const ORDERING_CHANNEL: u8 = #channel;
            const RELIABILITY: ::layline_codec::Reliability =
                ::layline_codec::Reliability::#reliability;
        }

        // Makes cargo rebuild the packet when its definition changes.
        const _: &[u8] = include_bytes!(#path);
    })
}
