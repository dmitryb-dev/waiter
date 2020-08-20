use proc_macro::{TokenStream};
use syn::{Ident, ItemStruct, Fields, Field, Type, Error, PathArguments, GenericArgument};
use syn::export::{TokenStream2, Span, ToTokens};
use syn::spanned::Spanned;

pub fn generate_component_impl(component: ItemStruct) -> TokenStream {
    let comp_name = &component.ident;
    let comp_generics = &component.generics;

    let dependencies_code = generate_dependencies_create_code(component.fields.iter().collect());
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
        impl #comp_generics waiter::Component for #comp_name #comp_generics {
            fn __waiter_create<P>(container: &mut Container<P>) -> Self {
                #dependencies_code
                return #comp_name #factory_code;
            }
            fn __waiter_inject_deferred<P>(container: &mut Container<P>, component: std::rc::Rc<Self>) {
                #deferred_dependencies_code
                #deferred_inject_code
            }
        }
    };

    return TokenStream::from(result);
}

fn generate_inject_dependencies_tuple(dep_number: usize) -> TokenStream2 {
    let dependencies: Vec<Ident> = (0..dep_number)
        .map(|i| Ident::new(&format!("dep_{}", i)[..], Span::call_site()))
        .collect();

    return quote::quote! {
        #(#dependencies),*
    };
}

fn generate_inject_dependencies_named(fields: Vec<&Field>) -> TokenStream2 {
    let dependencies: Vec<Ident> = (0..fields.len())
        .map(|i| Ident::new(&format!("dep_{}", i)[..], Span::call_site()))
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
            (i, Ident::new(&format!("{}", i)[..], Span::call_site()))
        } else {
            (i, f.ident.clone().unwrap())
        })
        .map(|(i, field_name)| {
            let dependency = Ident::new(&format!("dep_{}", i)[..], Span::call_site());
            quote::quote! { #field_name.init(#dependency); }
        })
        .collect();

    return quote::quote! {
        #(component.#dependencies_inject)*
    };
}

fn generate_dependencies_create_code(fields: Vec<&Field>) -> TokenStream2 {
    let dep_code_list: Vec<TokenStream2> = fields.iter()
        .enumerate()
        .map(|(i, f)| generate_dependency_create_code(&f.ty, i))
        .collect();

    return quote::quote! {
        #(#dep_code_list)*
    }
}

fn generate_dependency_create_code(typ: &Type, pos: usize) -> TokenStream2 {
    let dep_var_name = quote::format_ident!("dep_{}", pos);

    match typ {
        Type::Reference(type_ref) => {
            let referenced_type = &type_ref.elem;
            return quote::quote! {
                let #dep_var_name = unsafe {
                    (Provider::<#referenced_type>::get_ref(container) as *const #referenced_type)
                        .as_ref()
                        .unwrap()
                };
            };
        }
        Type::Path(path_type) => {
            let ptr_type = path_type.path.to_token_stream().to_string();

            if ptr_type.starts_with("std :: rc :: Rc <") {
                let referenced_type = &path_type.path.segments[2].arguments;
                return quote::quote! {
                    let #dep_var_name = Provider::#referenced_type::get(container);
                }
            }
            if ptr_type.starts_with("Rc <") {
                let referenced_type = &path_type.path.segments[0].arguments;
                return quote::quote! {
                    let #dep_var_name = Provider::#referenced_type::get(container);
                }
            }
            if ptr_type.starts_with("Box <") {
                let referenced_type = &path_type.path.segments[0].arguments;
                return quote::quote! {
                    let #dep_var_name = Provider::#referenced_type::create(container);
                }
            }
            if ptr_type.contains("Deferred <") {
                let deferred_arg = if ptr_type.starts_with("waiter :: Deferred <") {
                    &path_type.path.segments[1]
                } else if ptr_type.starts_with("Deferred <") {
                    &path_type.path.segments[0]
                } else {
                    panic!("Incorrect Deferred type: wrong crate")
                };

                let referenced_type = &deferred_arg.arguments;

                return quote::quote! {
                    let #dep_var_name = waiter::Deferred::#referenced_type::new();
                }
            }
            return Error::new(
                typ.span(),
                "Only &, Rc, Deferred, Component and #[prop(\"name\"] i64/f64/String/bool can be injected"
            ).to_compile_error()
        }
        _ => Error::new(
            typ.span(),
            "Only &, Rc, Component and #[prop(\"name\"] number/string can be injected"
        ).to_compile_error()
    }
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
        .map(|(i, t)| generate_dependency_create_code(&t, i))
        .collect();

    return quote::quote! {
        #(#dep_code_list)*
    }
}