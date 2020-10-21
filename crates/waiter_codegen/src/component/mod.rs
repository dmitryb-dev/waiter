use proc_macro::{TokenStream};
use syn::{Ident, ItemStruct, Fields, Field, Type, Error, PathArguments, GenericArgument, ItemImpl, ImplItem, Expr, ItemFn};
use syn::export::{TokenStream2, Span, ToTokens};
use component::injector::{RcInjector, DeferredInjector, BoxInjector, ConfigInjector,
                          Injector, PropInjector};
use syn::export::quote;
use syn::spanned::Spanned;
use attr_parser::parse_provides_attr;
use provider::generate_component_provider_impl_fn;
use component::type_to_inject::TypeToInject;

pub(crate) mod injector;
pub(crate) mod type_to_inject;

pub(crate) fn generate_component_for_impl(comp_impl: ItemImpl) -> Result<TokenStream, Error> {
    for item in &comp_impl.items {
        match item {
            ImplItem::Method(method) => {
                let provides_attr = method.attrs.iter()
                    .find(|attr| attr.path.to_token_stream().to_string() == "provides".to_string());

                if provides_attr.is_some() {
                    let provides_attr = provides_attr.unwrap();
                    let provides = if provides_attr.tokens.is_empty() {
                        parse_provides_attr(TokenStream::new())?
                    } else {
                        parse_provides_attr(
                            provides_attr.parse_args::<Expr>()?
                                .to_token_stream()
                                .into()
                        )?
                    };

                    let mut fn_tokens = method.sig.to_token_stream();
                    fn_tokens.extend(method.block.to_token_stream());
                    let item_fn = syn::parse::<ItemFn>(fn_tokens.into())?;

                    return generate_component_provider_impl_fn(
                        provides,
                        item_fn,
                        comp_impl.self_ty.to_token_stream().into()
                    )
                }
            },
            _ => {}
        }
    }
    Err(Error::new(comp_impl.span(), "Constructor with #[provides] attribute is not found"))
}

pub(crate) fn generate_component_for_struct(component: ItemStruct) -> Result<TokenStream, Error> {
    let comp_name = &component.ident;
    let comp_generics = &component.generics;
//    let comp_where = &component.generics.where_clause.unwrap().predicates[

    let dependencies_code = generate_dependencies_create_code(
        component.fields.iter()
            .map(|f| TypeToInject::from_field(f))
            .collect::<Result<Vec<_>, _>>()?
    );
    let deferred_dependencies_code = generate_deferred_dependencies_code(
        component.fields.iter().collect()
    )?;

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

    return Ok(result.into());
}

pub(crate) fn generate_inject_dependencies_tuple(dep_number: usize) -> TokenStream2 {
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

pub(crate) fn generate_dependencies_create_code(args: Vec<TypeToInject>) -> TokenStream2 {
    let dep_code_list: Vec<TokenStream2> = args.into_iter()
        .enumerate()
        .map(|(i, arg)| generate_dependency_create_code(arg, i)).collect();

    return quote::quote! {
        #(#dep_code_list)*
    }
}

fn generate_dependency_create_code(to_inject: TypeToInject, pos: usize) -> TokenStream2 {
    let dep_var_name = quote::format_ident!("dep_{}", pos);
    let type_path = to_inject.type_path.clone();

    let mut injectors: Vec<Box<dyn Injector>> = Vec::new();
    injectors.push(Box::new(DeferredInjector));
    injectors.push(Box::new(RcInjector));
    injectors.push(Box::new(BoxInjector));
    injectors.push(Box::new(ConfigInjector));
    injectors.push(Box::new(PropInjector));

    let inject_code = injectors.iter()
        .find_map(|injector| injector.generate_inject_code(
            &to_inject,
            &Ident::new("container", Span::call_site())
        ))
        .unwrap_or_else(|| quote::quote! { waiter_di::Provider::<#type_path>::create(container) });

    quote::quote! {
        let #dep_var_name = #inject_code;
    }
}

fn generate_deferred_dependencies_code(fields: Vec<&Field>) -> Result<TokenStream2, Error> {
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
        .map(|(i, t)| TypeToInject::from_type(t)
            .map(|to_inject| generate_dependency_create_code(to_inject, i))
        )
        .collect::<Result<Vec<_>, Error>>()?;

    return Ok(quote::quote! {
        #(#dep_code_list)*
    })
}