use crate::component::type_to_inject::TypeToInject;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{Error, Ident, PathArguments};

pub(crate) trait Injector {
    fn generate_inject_code(
        &self,
        to_inject: &TypeToInject,
        container: &Ident,
    ) -> Option<TokenStream2>;
}

pub(crate) struct WrcInjector;
impl Injector for WrcInjector {
    fn generate_inject_code(
        &self,
        to_inject: &TypeToInject,
        container: &Ident,
    ) -> Option<TokenStream2> {
        #[cfg(feature = "async")]
        const RC_FULL_TYPE: &str = "std :: sync :: Arc <";
        #[cfg(not(feature = "async"))]
        const RC_FULL_TYPE: &str = "std :: rc :: Rc <";

        #[cfg(feature = "async")]
        const RC_SHORT_TYPE: &str = "Arc <";
        #[cfg(not(feature = "async"))]
        const RC_SHORT_TYPE: &str = "Rc <";

        let referenced_type_opt = if to_inject.type_name.starts_with("waiter_di :: Wrc <")
            || to_inject.type_name.starts_with(RC_FULL_TYPE)
        {
            Some(get_type_arg(&to_inject.type_path.segments[2].arguments))
        } else if to_inject.type_name.starts_with("Wrc <")
            || to_inject.type_name.starts_with(RC_SHORT_TYPE)
        {
            Some(get_type_arg(&to_inject.type_path.segments[0].arguments))
        } else {
            None
        };

        referenced_type_opt.map(|ref_type| {
            quote::quote! {
                waiter_di::Provider::<#ref_type>::get(#container)
            }
        })
    }
}

pub(crate) struct BoxInjector;
impl Injector for BoxInjector {
    fn generate_inject_code(
        &self,
        to_inject: &TypeToInject,
        container: &Ident,
    ) -> Option<TokenStream2> {
        if to_inject.type_name.starts_with("Box <") {
            let referenced_type = get_type_arg(&to_inject.type_path.segments[0].arguments);
            return Some(quote::quote! {
                Box::new(waiter_di::Provider::<#referenced_type>::create(#container))
            });
        }

        None
    }
}

pub(crate) struct DeferredInjector;
impl Injector for DeferredInjector {
    fn generate_inject_code(
        &self,
        to_inject: &TypeToInject,
        _container: &Ident,
    ) -> Option<TokenStream2> {
        let referenced_type_opt = if to_inject.type_name.starts_with("waiter_di :: Deferred <") {
            Some(get_type_arg(&to_inject.type_path.segments[1].arguments))
        } else if to_inject.type_name.starts_with("Deferred <") {
            Some(get_type_arg(&to_inject.type_path.segments[0].arguments))
        } else {
            None
        };

        referenced_type_opt.map(|ref_type| {
            quote::quote! {
                waiter_di::Deferred::<#ref_type>::new()
            }
        })
    }
}

pub(crate) struct ConfigInjector;
impl Injector for ConfigInjector {
    fn generate_inject_code(
        &self,
        to_inject: &TypeToInject,
        container: &Ident,
    ) -> Option<TokenStream2> {
        if to_inject.type_name == *"Config".to_string()
            || to_inject.type_name == *"config :: Config".to_string()
        {
            return Some(quote::quote! { #container.config.clone() });
        }

        None
    }
}

pub(crate) struct PropInjector;
impl Injector for PropInjector {
    fn generate_inject_code(
        &self,
        to_inject: &TypeToInject,
        container: &Ident,
    ) -> Option<TokenStream2> {
        let (prop_name_opt, default_value_code) = if to_inject.prop_attr.is_some() {
            let prop_attr = to_inject.prop_attr.as_ref().unwrap();
            let prop_name_opt = prop_attr
                .name
                .clone()
                .or_else(|| to_inject.arg_name.clone().map(|e| e.to_string()));

            let default_value_code = prop_attr
                .default_value
                .as_ref()
                .map(|default_value| quote::quote! { .or_else(|| Some(#default_value)) })
                .unwrap_or_default();

            (prop_name_opt, default_value_code)
        } else {
            (
                to_inject
                    .arg_name
                    .clone()
                    .map(|name_ts| name_ts.to_string()),
                quote::quote! {},
            )
        };

        let base_types_extracted = prop_name_opt.and_then(|prop_name| {
            let (type_path, opt_extractor) = if to_inject.type_name.starts_with("Option <") {
                (get_type_arg(&to_inject.type_path.segments[0].arguments),
                 quote::quote! { }
                )
            } else {
                (to_inject.type_path.to_token_stream(),
                 quote::quote! { .expect(format!("Property \"{}\" not found", #prop_name).as_str()) }
                )
            };
            let type_name = type_path.to_string();

            let extractors: [Box<dyn PropExtractor>; 3] = [
                Box::new(SafeCastPropExtractor),
                Box::new(UnsafeCastPropExtractor),
                Box::new(AsCastPropExtractor)
            ];

            extractors.iter()
                .find_map(|extractor| extractor
                    .generate_extract_method(type_name.clone())
                    .map(|extract_method| {
                        let convert_code = extractor.generate_convert_code(
                            type_path.clone(),
                            type_name.clone(),
                            prop_name.to_string(),
                            quote::quote! { value }
                        );

                        quote::quote! {
                            #container.config.#extract_method(#prop_name)
                                .map(|value| #convert_code)
                                .ok()
                                #default_value_code
                                #opt_extractor
                        }
                    })
                )
        });

        base_types_extracted.or_else(|| {
            if to_inject.prop_attr.is_some() {
                let type_name = &to_inject.type_name;
                let type_path = &to_inject.type_path;
                Some(quote::quote! {
                    #container.config.clone().try_deserialize::<#type_path>()
                        .expect(format!("Can't parse config as '{}'", #type_name).as_str())
                })
            } else {
                None
            }
        })
    }
}

trait PropExtractor {
    fn generate_extract_method(&self, type_name: String) -> Option<TokenStream2>;
    fn generate_convert_code(
        &self,
        _type_path: TokenStream2,
        _type_name: String,
        _prop_name: String,
        extract_code: TokenStream2,
    ) -> TokenStream2 {
        extract_code
    }
}

struct SafeCastPropExtractor;
impl PropExtractor for SafeCastPropExtractor {
    fn generate_extract_method(&self, type_name: String) -> Option<TokenStream2> {
        match type_name.as_str() {
            "i128" | "u128" => Some(quote::quote! { get_int }),
            _ => None,
        }
    }

    fn generate_convert_code(
        &self,
        type_path: TokenStream2,
        _type_name: String,
        _prop_name: String,
        value: TokenStream2,
    ) -> TokenStream2 {
        quote::quote! { #value as #type_path }
    }
}

struct UnsafeCastPropExtractor;
impl PropExtractor for UnsafeCastPropExtractor {
    fn generate_extract_method(&self, type_name: String) -> Option<TokenStream2> {
        match type_name.as_str() {
            "i8" | "i16" | "i32" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => {
                Some(quote::quote! { get_int })
            }
            _ => None,
        }
    }

    fn generate_convert_code(
        &self,
        type_path: TokenStream2,
        type_name: String,
        prop_name: String,
        value: TokenStream2,
    ) -> TokenStream2 {
        quote::quote! {
            <#type_path as std::convert::TryFrom<i64>>::try_from(#value)
                .expect(format!("Can't parse prop '{}' as '{}'", #prop_name, #type_name).as_str())
        }
    }
}

struct AsCastPropExtractor;
impl PropExtractor for AsCastPropExtractor {
    fn generate_extract_method(&self, type_name: String) -> Option<TokenStream2> {
        match type_name.as_str() {
            "i64" => Some(quote::quote! { get_int }),
            "f64" | "f32" => Some(quote::quote! { get_float }),
            "String" => Some(quote::quote! { get_string }),
            "bool" => Some(quote::quote! { get_bool }),
            _ => None,
        }
    }

    fn generate_convert_code(
        &self,
        type_path: TokenStream2,
        _type_name: String,
        _prop_name: String,
        value: TokenStream2,
    ) -> TokenStream2 {
        quote::quote! {
            #value as #type_path
        }
    }
}

fn get_type_arg(arguments: &PathArguments) -> TokenStream2 {
    if let PathArguments::AngleBracketed(ab) = arguments {
        ab.args.to_token_stream()
    } else {
        Error::new(arguments.span(), "Unsupported type argument").to_compile_error()
    }
}
