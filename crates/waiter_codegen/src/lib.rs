extern crate proc_macro;
extern crate syn;
extern crate regex;

use proc_macro::TokenStream;
use syn::*;
use component::generate_component_impl;
use provider::*;
use std::str::FromStr;
use regex::Regex;
use attr_parser::{ProvidesAttr, parse_provides_attr};
use syn::spanned::Spanned;

mod component;
mod provider;
mod attr_parser;

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let comp = syn::parse::<ItemStruct>(item.clone()).unwrap();

    let mut res: TokenStream = remove_prop_annotation(&item);

    res.extend(generate_component_impl(comp.clone()));
    res.extend(generate_component_provider_impl_struct(comp.clone()));

    return res;
}

#[proc_macro_attribute]
pub fn provides(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ProvidesAttr { profiles } = match parse_provides_attr(attr) {
        Ok(attr) => attr,
        Err(error) => return error.to_compile_error().into()
    };

    let mut res = remove_prop_annotation(&item);

    let impl_block = syn::parse::<ItemImpl>(item.clone());
    if impl_block.is_ok() {
        res.extend(generate_interface_provider_impl(profiles, impl_block.unwrap().clone()));
    } else {
        let fn_block = syn::parse::<ItemFn>(item.clone())
            .expect("#[provides] must be used only on impl blocks and factory functions");

        res.extend(generate_component_provider_impl_fn(profiles, fn_block.clone()));
    }

    return res;
}

#[proc_macro_attribute]
pub fn wrapper(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let wrapper = parse_macro_input!(item as ItemStruct);

    let type_to_wrap = if let Fields::Unnamed(fields) = &wrapper.fields {
        let field = fields.unnamed.first();
        if field.is_none() {
            return TokenStream::from(
                Error::new(wrapper.span(), "Struct annotated #[wrapper] must have exactly one field")
                    .to_compile_error()
            );
        }

        field.unwrap().ty.clone()
    } else {
        return TokenStream::from(
            Error::new(wrapper.span(), "Only tuple like struct supported for #[wrapper]")
                .to_compile_error()
        );
    };

    let wrapper_name = &wrapper.ident;

    return TokenStream::from(quote::quote! {
        #wrapper
        impl Deref for #wrapper_name {
            type Target = #type_to_wrap;

            fn deref(&self) -> &Self::Target {
                return &self.0;
            }
        }
    });
}

fn remove_prop_annotation(item: &TokenStream) -> TokenStream {
    TokenStream::from_str(
        Regex::new(r"#\[prop.*?]").unwrap()
            .replace_all(item.to_string().as_str(), "")
            .as_ref()
    ).unwrap_or_default()
}