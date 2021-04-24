use syn::{Path, Error, Attribute};
use syn::punctuated::Punctuated;
use syn::parse::Parser;
use syn::token::{Comma};
use syn::{LitStr, ExprAssign};
use syn::parse_macro_input::parse;
use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
use quote::ToTokens;

pub(crate) struct ProvidesAttr {
    pub profiles: Vec<Path>
}

pub(crate) fn parse_provides_attr(attr: TokenStream) -> Result<ProvidesAttr, Error> {
    let profiles_syn = <Punctuated<Path, Comma>>::parse_terminated.parse(attr)?;

    let profiles: Vec<Path> = profiles_syn
        .iter()
        .map(|p| p.clone())
        .collect();

    Ok(ProvidesAttr { profiles })
}


#[derive(Clone)]
pub(crate) struct PropAttr {
    pub(crate) name: Option<String>,
    pub(crate) default_value: Option<TokenStream2>
}

pub(crate) fn parse_prop_attr(attr: &Attribute) -> Result<PropAttr, Error> {
    if attr.tokens.is_empty() {
        Ok(PropAttr { name: None, default_value: None })
    } else {
        attr.parse_args::<ExprAssign>()
            .and_then(|with_default| {
                let name = parse::<LitStr>(with_default.left.to_token_stream().into())?;

                Ok(PropAttr {
                    name: Some(name.value()),
                    default_value: Some(with_default.right.to_token_stream())
                })
            })
            .or_else(|_| {
                Ok(PropAttr {
                    name: Some(attr.parse_args::<LitStr>()?.value()),
                    default_value: None
                })
            })
    }
}