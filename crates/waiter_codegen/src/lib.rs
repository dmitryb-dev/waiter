use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
use quote::ToTokens;
use syn::*;
use component::{generate_component_for_struct, generate_component_for_impl};
use provider::*;
use attr_parser::{parse_provides_attr};
use syn::spanned::Spanned;
use syn::punctuated::Punctuated;
use syn::token::Comma;

mod component;
mod provider;
mod attr_parser;

#[proc_macro_attribute]
pub fn module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    component(_attr, item)
}

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut res: TokenStream = remove_attrs(item.clone());

    let comp = syn::parse::<ItemStruct>(item.clone());
    if comp.is_ok() {
        let comp = comp.unwrap();
        res.extend(unwrap(generate_component_for_struct(comp.clone())));
        res.extend(generate_component_provider_impl_struct(comp.clone()));
        return res;
    }

    let impl_block = syn::parse::<ItemImpl>(item.clone())
        .expect("#[component]/#[module] cant be used only on struct or impls");
    res.extend(unwrap(generate_component_for_impl(impl_block.clone())));
    return res;
}

#[proc_macro_attribute]
pub fn provides(attr: TokenStream, item: TokenStream) -> TokenStream {
    let provides_attr = match parse_provides_attr(attr) {
        Ok(attr) => attr,
        Err(error) => return error.to_compile_error().into()
    };

    let mut res = remove_attrs(item.clone());

    let impl_block = syn::parse::<ItemImpl>(item.clone());
    if impl_block.is_ok() {
        res.extend(generate_interface_provider_impl(provides_attr, impl_block.unwrap().clone()));
        return res;
    }

    let fn_block = syn::parse::<ItemFn>(item.clone())
        .expect("#[provides] must be used only on impl blocks and factory functions");
    res.extend(unwrap(generate_component_provider_impl_fn(
        provides_attr,
        fn_block.clone(),
        TokenStream2::new()
    )));
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
        impl std::ops::Deref for #wrapper_name {
            type Target = #type_to_wrap;

            fn deref(&self) -> &Self::Target {
                return &self.0;
            }
        }
    });
}

fn remove_attrs(item: TokenStream) -> TokenStream {
    fn attr_filter(attr: &Attribute) -> bool {
        let attr_name = attr.path.to_token_stream().to_string();
        attr_name.as_str() != "prop" && attr_name.as_str() != "provides"
    }

    let item = syn::parse::<Item>(item).unwrap();

    let item = match item {
        Item::Fn(mut fn_) => {
            fn_.attrs.retain(attr_filter);
            fn_.sig.inputs.iter_mut()
                .for_each(|arg| if let FnArg::Typed(path) = arg {
                    path.attrs.retain(attr_filter);
                });
            Item::Fn(fn_)
        },
        Item::Impl(impl_) => {
            let mut impl_filtered = impl_.clone();
            impl_filtered.items.clear();

            for impl_item in impl_.items {
                let impl_item = match impl_item {
                    ImplItem::Method(method) => {
                        let mut method_filtered = method.clone();
                        method_filtered.attrs.retain(attr_filter);
                        method_filtered.sig.inputs.clear();

                        for fn_arg in method.sig.inputs {
                            let filtered = match fn_arg {
                                FnArg::Typed(mut typed) => {
                                    typed.attrs.retain(attr_filter);
                                    FnArg::Typed(typed)
                                },
                                other => other
                            };
                            method_filtered.sig.inputs.push(filtered);
                        }

                        ImplItem::Method(method_filtered)
                    },
                    other => other
                };
                impl_filtered.items.push(impl_item)
            };

            Item::Impl(impl_filtered)
        },
        Item::Struct(struct_) => {
            let mut struct_filtered = struct_.clone();

            fn filter_fields(fields: Punctuated<Field, Comma>) -> Punctuated<Field, Comma> {
                let mut fields_filtered = fields.clone();
                fields_filtered.clear();

                for mut field in fields {
                    field.attrs.retain(attr_filter);
                    fields_filtered.push(field);
                }

                fields_filtered
            }

            struct_filtered.fields = match struct_.fields {
                Fields::Named(mut fields) => {
                    fields.named = filter_fields(fields.named.clone());
                    Fields::Named(fields)
                },
                Fields::Unnamed(mut fields) => {
                    fields.unnamed = filter_fields(fields.unnamed.clone());
                    Fields::Unnamed(fields)
                },
                other => other
            };

            Item::Struct(struct_filtered)
        },
        other => other
    };

    item.to_token_stream().into()
}

fn unwrap(result: Result<TokenStream>) -> TokenStream {
    match result {
        Ok(result) => result,
        Err(err) => err.to_compile_error().into()
    }
}