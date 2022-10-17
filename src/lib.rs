use std::collections::HashMap;

use proc_macro2::{Ident, Literal};
use quote::{format_ident, quote};
use syn::{parse::Parse, punctuated::Punctuated, DeriveInput, Token};

extern crate proc_macro;

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
enum LanguageIdentifier {
    ISO639_1(String),
    ISO639_3(String),
    NLLB(String),
}

impl Parse for LanguageIdentifier {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let identifier: Ident = input.parse()?;
        let _eq: Token!(=) = input.parse()?;
        let inner: Literal = input.parse()?;

        let inner_source = inner.to_string();
        let mut inner = inner_source.chars();
        inner.next();
        inner.next_back();
        let inner = inner.as_str();

        match identifier.to_string().as_str() {
            "iso639_1" => Ok(Self::ISO639_1(inner.into())),
            "iso639_3" => Ok(Self::ISO639_3(inner.into())),
            "nllb" => Ok(Self::NLLB(inner.into())),
            other => Err(input.error(format!(
                "should be `iso639_1`, `iso639_3` or `nllb`, parsed: {other}"
            ))),
        }
    }
}

#[derive(Default, Debug)]
struct LanguageIdentifiers {
    iso_639_1: Option<String>,
    iso_639_3: Option<String>,
    nllb: Option<String>,
}

impl Parse for LanguageIdentifiers {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ids: Punctuated<LanguageIdentifier, Token!(,)> =
            input.parse_terminated(LanguageIdentifier::parse)?;

        ids.into_iter().try_fold(Self::default(), |mut ids, id| {
            match id {
                LanguageIdentifier::ISO639_1(id) => ids.iso_639_1 = Some(id),
                LanguageIdentifier::ISO639_3(id) => ids.iso_639_3 = Some(id),
                LanguageIdentifier::NLLB(id) => ids.nllb = Some(id),
            };
            Ok(ids)
        })
    }
}

fn into_helper(
    name: proc_macro2::TokenStream,
    source: &HashMap<Ident, LanguageIdentifiers>,
    getter: impl Fn(&LanguageIdentifiers) -> Option<&String>,
) -> proc_macro2::TokenStream {
    let source = source
        .iter()
        .filter_map(|(ident, ids)| getter(ids).map(|id| (ident, id)))
        .map(|(ident, id)| {
            let ident = format_ident!("{}", ident);
            quote! { Self::#ident => #id }
        })
        .collect::<Vec<_>>();

    quote! {
        impl Language {
            #[allow(unreachable_code)]
            pub fn #name(&self) -> Option<&'static str> {
                let result = match self {
                    #( #source, )*
                    _ => return None,
                };
                Some(result)
            }
        }
    }
}

fn from_helper(
    name: proc_macro2::TokenStream,
    source: &HashMap<Ident, LanguageIdentifiers>,
    getter: impl Fn(&LanguageIdentifiers) -> Option<&String>,
) -> proc_macro2::TokenStream {
    let source = source
        .iter()
        .filter_map(|(ident, ids)| getter(ids).map(|id| (ident, id)))
        .map(|(ident, id)| {
            let ident = format_ident!("{}", ident);
            quote! { #id => Self::#ident }
        })
        .collect::<Vec<_>>();

    quote! {
        impl Language {
            #[allow(unreachable_code)]
            pub fn #name(source: &str) -> Option<Self> {
                let result = match source {
                    #( #source, )*
                    _ => return None,
                };
                Some(result)
            }
        }
    }
}

#[proc_macro_derive(LanguageCodes, attributes(language))]
pub fn codes_setter(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(tokens).unwrap();

    let mut gids = HashMap::new();

    if let syn::Data::Enum(e) = input.data {
        for variant in e.variants {
            let attribute = match variant.attrs.first() {
                Some(attr) => attr,
                None => continue,
            };

            let ids: LanguageIdentifiers = match attribute.parse_args() {
                Ok(ids) => ids,
                Err(e) => return e.into_compile_error().into(),
            };

            gids.insert(variant.ident, ids);
        }
    }

    let into_1 = into_helper(quote! { get_iso639_1 }, &gids, |ids| ids.iso_639_1.as_ref());
    let into_3 = into_helper(quote! { get_iso639_3 }, &gids, |ids| ids.iso_639_3.as_ref());
    let into_nllb = into_helper(quote! { get_nllb }, &gids, |ids| ids.nllb.as_ref());

    let from_1 = from_helper(quote! { from_iso639_1 }, &gids, |ids| ids.iso_639_1.as_ref());
    let from_3 = from_helper(quote! { from_iso639_3 }, &gids, |ids| ids.iso_639_3.as_ref());
    let from_nllb = from_helper(quote! { from_nllb }, &gids, |ids| ids.nllb.as_ref());

    quote! {
        #into_1
        #into_3
        #into_nllb

        #from_1
        #from_3
        #from_nllb
    }
    .into()
}
