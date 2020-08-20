extern crate proc_macro;
extern crate waiter_core;
extern crate syn;

use proc_macro::TokenStream;
use syn::*;
use component::generate_component_impl;

mod component;
mod provider;

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let comp = syn::parse::<ItemStruct>(item.clone()).unwrap();

    let mut res = item.clone();
    res.extend(generate_component_impl(comp));

    return res;
}