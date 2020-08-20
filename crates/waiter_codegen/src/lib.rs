extern crate proc_macro;
extern crate waiter_core;
extern crate syn;

use proc_macro::TokenStream;
use syn::*;
use component::generate_component_impl;
use provider::{generate_component_provider_impl, generate_interface_provider_impl};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::parse::Parser;

mod component;
mod provider;

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let comp = syn::parse::<ItemStruct>(item.clone()).unwrap();

    let mut res = item.clone();
    res.extend(generate_component_impl(comp.clone()));
    res.extend(generate_component_provider_impl(comp.clone()));

    return res;
}

#[proc_macro_attribute]
pub fn provides(attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_block = syn::parse::<ItemImpl>(item.clone()).unwrap();

    let profiles = <Punctuated<Path, Comma>>::parse_terminated.parse(attr)
        .expect("Can't parse profiles");
    let profiles: Vec<&Path> = profiles
        .iter()
        .collect();

    let mut res = item.clone();
    res.extend(generate_interface_provider_impl(profiles, impl_block.clone()));

    return res;
}