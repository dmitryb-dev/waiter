use syn::{Path, Ident, Type, Field, Pat, FnArg, Error, Attribute, PathArguments};
use syn::export::{TokenStream2, ToTokens};
use syn::spanned::Spanned;
use attr_parser::{PropAttr, parse_prop_attr};

#[derive(Clone)]
pub(crate) struct TypeToInject {
    type_name: String,
    pub(crate) type_path: Path,
    arg_name: Option<TokenStream2>,
    prop_attr: Option<PropAttr>
}


impl TypeToInject {
    pub(crate) fn from_type(type_: &Type) -> Result<Self, Error> {
        Ok(Self {
            type_name: type_.to_token_stream().to_string(),
            type_path: Self::parse_path(type_)?,
            arg_name: None,
            prop_attr: None
        })
    }
    pub(crate) fn from_field(field: &Field) -> Result<Self, Error> {
        Ok(Self {
            type_name: field.ty.to_token_stream().to_string(),
            type_path: Self::parse_path(&field.ty)?,
            arg_name: field.ident.clone().map(|name| name.to_token_stream()),
            prop_attr: Self::parse_attr(&field.attrs)?
        })
    }
    pub(crate) fn from_fn_arg(arg: FnArg) -> Result<Self, Error> {
        let typed = if let FnArg::Typed(typed) = &arg {
          typed
        } else {
            return Err(Error::new(arg.span(), "Unsupported argument type"))
        };

        let arg_name = if let Pat::Ident(pat_ident) = *typed.pat.clone() {
            Some(pat_ident.ident.to_token_stream())
        } else {
            return Err(Error::new(arg.span(), "Unsupported argument name"))
        };

        Ok(Self {
            type_name: typed.ty.to_token_stream().to_string(),
            type_path: Self::parse_path(&typed.ty)?,
            arg_name,
            prop_attr: Self::parse_attr(&typed.attrs)?
        })
    }

    fn parse_attr(attrs: &Vec<Attribute>) -> Result<Option<PropAttr>, Error> {
        let prop_attr = attrs.iter()
            .find(|attr| attr.path.to_token_stream().to_string() == "prop".to_string());

        if prop_attr.is_some() {
            return parse_prop_attr(prop_attr.unwrap())
                .map(|parsed| Some(parsed));
        }
        Ok(None)
    }

    fn parse_path(type_: &Type) -> Result<Path, Error> {
        if let Type::Path(path_type) = type_ {
            Ok(path_type.path.clone())
        } else {
            Err(Error::new(type_.span(), "Unsupported type"))
        }
    }
}



pub(crate) trait Injector {
    fn generate_inject_code(
        &self,
        to_inject: &TypeToInject,
        container: &Ident
    ) -> Option<TokenStream2>;
}

pub(crate) struct RcInjector;
impl Injector for RcInjector {
    fn generate_inject_code(
        &self,
        to_inject: &TypeToInject,
        container: &Ident
    ) -> Option<TokenStream2> {
        let referenced_type_opt = if to_inject.type_name.starts_with("std :: rc :: Rc <") {
            Some(get_type_arg(&to_inject.type_path.segments[2].arguments))
        } else if to_inject.type_name.starts_with("Rc <") {
            Some(get_type_arg(&to_inject.type_path.segments[0].arguments))
        } else {
            None
        };

        referenced_type_opt.map(|ref_type| quote::quote! {
            waiter_di::Provider::<#ref_type>::get(#container)
        })
    }
}


pub(crate) struct BoxInjector;
impl Injector for BoxInjector {
    fn generate_inject_code(
        &self,
        to_inject: &TypeToInject,
        container: &Ident
    ) -> Option<TokenStream2> {
        if to_inject.type_name.starts_with("Box <") {
            let referenced_type = get_type_arg(&to_inject.type_path.segments[0].arguments);
            return Some(quote::quote! {
                waiter_di::Provider::<#referenced_type>::create(#container)
            })
        }

        None
    }
}


pub(crate) struct DeferredInjector;
impl Injector for DeferredInjector {
    fn generate_inject_code(
        &self,
        to_inject: &TypeToInject,
        _container: &Ident
    ) -> Option<TokenStream2> {
        let referenced_type_opt = if to_inject.type_name.starts_with("waiter_di :: Deferred <") {
            Some(get_type_arg(&to_inject.type_path.segments[1].arguments))
        } else if to_inject.type_name.starts_with("Deferred <") {
            Some(get_type_arg(&to_inject.type_path.segments[0].arguments))
        } else {
            None
        };

        referenced_type_opt.map(|ref_type| quote::quote! {
            waiter_di::Deferred::<#ref_type>::new()
        })
    }
}


pub(crate) struct ConfigInjector;
impl Injector for ConfigInjector {
    fn generate_inject_code(
        &self,
        to_inject: &TypeToInject,
        container: &Ident
    ) -> Option<TokenStream2> {
        if to_inject.type_name == "Config".to_string()
            || to_inject.type_name == "config :: Config".to_string() {
            return Some(quote::quote! { #container.config.clone() })
        }

        None
    }
}


pub(crate) struct PropInjector;
impl Injector for PropInjector {
    fn generate_inject_code(
        &self,
        to_inject: &TypeToInject,
        container: &Ident
    ) -> Option<TokenStream2> {
        let (prop_name_opt, default_value_code) = if to_inject.prop_attr.is_some() {
            let prop_attr = to_inject.prop_attr.clone().unwrap();
            let prop_name_opt = prop_attr.name.clone()
                .or(to_inject.arg_name.clone()
                    .map(|name_ts| name_ts.to_string())
                );

            let default_value_code = prop_attr.default_value.clone()
                .map(|default_value| quote::quote! { .or_else(|| Some(#default_value)) })
                .unwrap_or(quote::quote! {});

            (prop_name_opt, default_value_code)
        } else {
            (to_inject.arg_name.clone()
                 .map(|name_ts| name_ts.to_string()),
             quote::quote! {}
            )
        };

        let base_types_extracted = prop_name_opt.and_then(|prop_name_tokens| {
            let prop_name = prop_name_tokens.to_string();

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

            let mut extractors: Vec<Box<dyn PropExtractor>> = Vec::new();
            extractors.push(Box::new(SafeCastPropExtractor));
            extractors.push(Box::new(UnsafeCastPropExtractor));
            extractors.push(Box::new(AsCastPropExtractor));

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

        base_types_extracted
            .or_else(|| {
                if to_inject.prop_attr.is_some() {
                    let type_name = to_inject.type_name.clone();
                    let type_path = to_inject.type_path.clone();
                    Some(quote::quote! {
                        #container.config.clone().try_into::<#type_path>()
                            .expect(format!("Can't parse config as '{}'", #type_name).as_str())
                    })
                } else {
                    None
                }
            })
    }
}

trait PropExtractor {
    fn generate_extract_method(&self, type_name: String, ) -> Option<TokenStream2>;
    fn generate_convert_code(
        &self,
        _type_path: TokenStream2,
        _type_name: String,
        _prop_name: String,
        extract_code: TokenStream2
    ) -> TokenStream2 {
        extract_code
    }
}

struct SafeCastPropExtractor;
impl PropExtractor for SafeCastPropExtractor {
    fn generate_extract_method(&self, type_name: String) -> Option<TokenStream2> {
        match type_name.as_str() {
            "i128" | "u128" => Some(quote::quote! { get_int }),
            _ => None
        }
    }

    fn generate_convert_code(
        &self,
        type_path: TokenStream2,
        _type_name: String,
        _prop_name: String,
        value: TokenStream2
    ) -> TokenStream2 {
        quote::quote! { #type_path::from(#value) }
    }
}

struct UnsafeCastPropExtractor;
impl PropExtractor for UnsafeCastPropExtractor {
    fn generate_extract_method(&self, type_name: String) -> Option<TokenStream2> {
        match type_name.as_str() {
            "i8" | "i16" | "i32" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" =>
                Some(quote::quote! { get_int }),
            _ => None
        }
    }

    fn generate_convert_code(
        &self,
        type_path: TokenStream2,
        type_name: String,
        prop_name: String,
        value: TokenStream2
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
            "String" => Some(quote::quote! { get_str }),
            "bool" => Some(quote::quote! { get_bool }),
            _ => None
        }
    }

    fn generate_convert_code(
        &self,
        type_path: TokenStream2,
        _type_name: String,
        _prop_name: String,
        value: TokenStream2
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