use proc_macro::{TokenStream};
use syn::{Ident, ItemStruct, Fields, Field, Type, Error, PathArguments, GenericArgument, LitStr,
          FnArg, Attribute, Pat};
use syn::export::{TokenStream2, Span, ToTokens};
use syn::spanned::Spanned;

pub struct Argument {
    attrs: Vec<Attribute>,
    name: Option<String>,
    type_: Type
}

pub struct PropertyAttr {
    is_annotated: bool,
    name: Option<String>
}

impl Argument {
    fn from_type(type_: &Type) -> Self {
        Self { attrs: vec!(), name: None, type_: type_.clone() }
    }
    fn from_field(field: Field) -> Self {
        Self { attrs: field.attrs, name: field.ident.map(|ident| ident.to_string()), type_: field.ty}
    }
    pub fn from_fn_arg(arg: FnArg) -> Self {
        if let FnArg::Typed(typed) = arg {
            let name = if let Pat::Ident(pat_ident) = *typed.pat {
                pat_ident.ident.to_string()
            } else {
                panic!("Unsupported argument name")
            };
            Self { attrs: typed.attrs, name: Some(name), type_: *typed.ty}
        } else {
            panic!("Unsupported argument type")
        }
    }
    fn prop_attr(&self) -> Option<PropertyAttr> {
        let prop_attr = self.attrs.iter()
            .find(|attr| attr.path.to_token_stream().to_string().eq(&"prop".to_string()));

        if prop_attr.is_some() {
            if prop_attr.unwrap().tokens.is_empty() {
                Some(PropertyAttr { name: None, is_annotated: true })
            } else {
                let name = prop_attr.unwrap()
                    .parse_args::<LitStr>().expect("Only string literals supported for #[prop(\"name\"]")
                    .value();

                Some(PropertyAttr { name: Some(name), is_annotated: true })
            }
        } else {
            self.name.clone()
                .map(|name| PropertyAttr { name: Some(name), is_annotated: false })
        }
    }
}

pub fn generate_component_impl(component: ItemStruct) -> TokenStream {
    let comp_name = &component.ident;
    let comp_generics = &component.generics;

    let dependencies_code = generate_dependencies_create_code(
        component.fields.iter()
            .map(|f| Argument::from_field(f.clone()))
            .collect()
    );
    let deferred_dependencies_code = generate_deferred_dependencies_code(component.fields.iter().collect());

    let (factory_code, deferred_inject_code) = match component.fields {
        Fields::Named(fields) => (
            generate_inject_dependencies_named(fields.named.iter().collect()),
            generate_inject_deferred(fields.named.iter().collect(), false)
        ),
        Fields::Unnamed(fields) => (
            generate_inject_dependencies_tuple(fields.unnamed.len()),
            generate_inject_deferred(fields.unnamed.iter().collect(), true)
        ),
        Fields::Unit => (
            generate_inject_dependencies_tuple(0),
            generate_inject_deferred(vec!(), true)
        ),
    };


    let result = quote::quote! {
        impl #comp_generics waiter_di::Component for #comp_name #comp_generics {
            fn __waiter_create<P>(container: &mut waiter_di::Container<P>) -> Self {
                #dependencies_code
                return #comp_name #factory_code;
            }
            fn __waiter_inject_deferred<P>(container: &mut waiter_di::Container<P>, component: &Self) {
                #deferred_dependencies_code
                #deferred_inject_code
            }
        }
    };

    return TokenStream::from(result);
}

pub fn generate_inject_dependencies_tuple(dep_number: usize) -> TokenStream2 {
    let dependencies: Vec<Ident> = (0..dep_number)
        .map(|i| Ident::new(format!("dep_{}", i).as_str(), Span::call_site()))
        .collect();

    return quote::quote! {
        (#(#dependencies),*)
    };
}

fn generate_inject_dependencies_named(fields: Vec<&Field>) -> TokenStream2 {
    let dependencies: Vec<Ident> = (0..fields.len())
        .map(|i| Ident::new(format!("dep_{}", i).as_str(), Span::call_site()))
        .collect();

    let field_names: Vec<&Ident> = fields.iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();

    return quote::quote! {
        {#(#field_names: #dependencies),*}
    };
}

fn generate_inject_deferred(fields: Vec<&Field>, is_tuple: bool) -> TokenStream2 {
    let dependencies_inject: Vec<TokenStream2> = fields.iter()
        .enumerate()
        .filter(|(_, f)| {
            if let Type::Path(path_type) = &f.ty {
                let ptr_type = path_type.path.to_token_stream().to_string();

                return ptr_type.starts_with("waiter :: Deferred <") || ptr_type.starts_with("Deferred <")
            }
            return false;
        })
        .map(|(i, f)| if is_tuple {
            (i, Ident::new(format!("{}", i).as_str(), Span::call_site()))
        } else {
            (i, f.ident.clone().unwrap())
        })
        .map(|(i, field_name)| {
            let dependency = Ident::new(format!("dep_{}", i).as_str(), Span::call_site());
            quote::quote! { #field_name.init(#dependency); }
        })
        .collect();

    return quote::quote! {
        #(component.#dependencies_inject)*
    };
}

pub fn generate_dependencies_create_code(args: Vec<Argument>) -> TokenStream2 {
    let dep_code_list: Vec<TokenStream2> = args.iter()
        .enumerate()
        .map(|(i, arg)| generate_dependency_create_code(arg.clone(), i)).collect();

    return quote::quote! {
        #(#dep_code_list)*
    }
}

fn generate_dependency_create_code(arg: &Argument, pos: usize) -> TokenStream2 {
    let dep_var_name = quote::format_ident!("dep_{}", pos);

    match &arg.type_ {
        Type::Path(path_type) => {
            let type_name = path_type.path.to_token_stream().to_string();

            if type_name.starts_with("std :: rc :: Rc <") {
                let referenced_type = &path_type.path.segments[2].arguments;
                return quote::quote! {
                    let #dep_var_name = waiter_di::Provider::#referenced_type::get(container);
                }
            }
            if type_name.starts_with("Rc <") {
                let referenced_type = &path_type.path.segments[0].arguments;
                return quote::quote! {
                    let #dep_var_name = waiter_di::Provider::#referenced_type::get(container);
                }
            }
            if type_name.starts_with("Box <") {
                let referenced_type = &path_type.path.segments[0].arguments;
                return quote::quote! {
                    let #dep_var_name = waiter_di::Provider::#referenced_type::create(container);
                }
            }
            if type_name.contains("Deferred <") {
                let deferred_arg = if type_name.starts_with("waiter :: Deferred <") {
                    &path_type.path.segments[1]
                } else if type_name.starts_with("Deferred <") {
                    &path_type.path.segments[0]
                } else {
                    panic!("Incorrect Deferred type: wrong crate")
                };

                let referenced_type = &deferred_arg.arguments;

                return quote::quote! {
                    let #dep_var_name = waiter_di::Deferred::#referenced_type::new();
                }
            }
            if type_name.eq(&"Config".to_string()) || type_name.eq(&"config :: Config".to_string()) {
                return quote::quote! {
                    let #dep_var_name = container.config.clone();
                }
            }


            if arg.prop_attr().is_some() {
                let prop_attr = arg.prop_attr().unwrap();

                if prop_attr.name.is_none() {
                    return quote::quote! {
                        let #dep_var_name = container.config.clone().try_into::<#path_type>()
                            .expect(format!("Can't parse config as {}", #type_name).as_str());
                    };
                }

                let prop_name = prop_attr.name.unwrap();

                let config_safe_number_cast_method = match type_name.as_str() {
                    "i128" | "u128" => Some(Ident::new("get_int", Span::call_site())),
                    _ => None
                };
                if config_safe_number_cast_method.is_some() {
                    return quote::quote! {
                        let #dep_var_name = #path_type::from(
                             container.config.#config_safe_number_cast_method(#prop_name)
                                .expect(format!("Property {} not found", #prop_name).as_str())
                        );
                    }
                }

                let (config_unsafe_number_cast_method, config_ret_type) = match type_name.as_str() {
                    "i8" | "i16" | "i32" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" =>
                        (Some(Ident::new("get_int", Span::call_site())), quote::quote! { i64 }),
                    _ => (None, quote::quote! {})
                };
                if config_unsafe_number_cast_method.is_some() {
                    return quote::quote! {
                        let #dep_var_name: #path_type = <#path_type as std::convert::TryFrom<#config_ret_type>>::try_from(
                             container.config.#config_unsafe_number_cast_method(#prop_name)
                                .expect(format!("Property {} not found", #prop_name).as_str())
                        ).expect(format!("Can't parse prop {} as {}", #prop_name, #type_name).as_str());
                    }
                }

                let config_method = match type_name.as_str() {
                    "i64" => Some(Ident::new("get_int", Span::call_site())),
                    "f64" | "f32" => Some(Ident::new("get_float", Span::call_site())),
                    "String" => Some(Ident::new("get_str", Span::call_site())),
                    "bool" => Some(Ident::new("get_bool", Span::call_site())),
                    _ => None
                };
                if config_method.is_some() {
                    return quote::quote! {
                        let #dep_var_name = container.config.#config_method(#prop_name)
                            .expect(format!("Property {} not found", #prop_name).as_str())
                            as #path_type;
                    }
                }

                if prop_attr.is_annotated {
                    panic!("Unsupported property type");
                }
            }

            return quote::quote! {
                let #dep_var_name = *waiter_di::Provider::<#path_type>::create(container);
            }
        }
        _ => {}
    }

    Error::new(
        arg.type_.span(),
        "Only Rc, Box, Deferred, Component, Config and #[prop(\"name\"] number/String/bool can be injected"
    ).to_compile_error()
}

fn generate_deferred_dependencies_code(fields: Vec<&Field>) -> TokenStream2 {
    let dep_code_list: Vec<TokenStream2> = fields.iter()
        .enumerate()
        .map(|(i, f)| {
            if let Type::Path(path_type) = &f.ty {
                let ptr_type = path_type.path.to_token_stream().to_string();

                let mut generic_args = None;
                if ptr_type.starts_with("waiter :: Deferred <") {
                    if let PathArguments::AngleBracketed(typ) = &path_type.path.segments[1].arguments {
                        generic_args = Some(&typ.args);
                    }
                } else if ptr_type.starts_with("Deferred <") {
                    if let PathArguments::AngleBracketed(typ) = &path_type.path.segments[0].arguments {
                        generic_args = Some(&typ.args);
                    }
                }
                if generic_args.is_some() {
                    if let GenericArgument::Type(typ) = generic_args.unwrap()
                        .first()
                        .expect("Expected <type> arg for Deferred type")
                    {
                        return (i, Some(typ));
                    }
                }
            }
            return (i, None);
        })
        .filter(|(_, opt_arg)| opt_arg.is_some())
        .map(|(i, opt_arg)| (i, opt_arg.unwrap()))
        .map(|(i, t)| generate_dependency_create_code(&Argument::from_type(t), i))
        .collect();

    return quote::quote! {
        #(#dep_code_list)*
    }
}