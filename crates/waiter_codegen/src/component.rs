use proc_macro::{TokenStream};
use syn::{Ident, ItemStruct, Fields, Field, Type, Error};
use syn::export::{TokenStream2, Span};
use syn::spanned::Spanned;

pub fn generate_component_impl(component: ItemStruct) -> TokenStream {
    let comp_name = &component.ident;
    let comp_generics = &component.generics;

    let dependencies_code = generate_dependencies_create_code(component.fields.iter().collect());

    let factory_code = match component.fields {
        Fields::Named(fields) =>
            generate_inject_dependencies_named(fields.named.iter().collect()),
        Fields::Unnamed(fields) =>
            generate_inject_dependencies_tuple(fields.unnamed.len()),
        Fields::Unit =>
            generate_inject_dependencies_tuple(0)
    };


    let result = quote::quote! {
        impl #comp_generics waiter::Component for #comp_name #comp_generics {
            fn __waiter_create(container: &mut Container<profiles::Default>) -> Self {
                #dependencies_code
                return #comp_name #factory_code;
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
        #(#dependencies)*
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
        {#(#field_names: #dependencies)*}
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

    if let Type::Reference(type_ref) = typ {
        let referenced_type = &type_ref.elem;
        quote::quote! {
            let #dep_var_name = unsafe {
                (Provider::<#referenced_type>::get_ref(container) as *const #referenced_type)
                    .as_ref()
                    .unwrap()
            };
        }
    } else {
        Error::new(
            typ.span(),
            "Only &, Rc, Component and #[prop(\"name\"] number/string can be injected"
        ).to_compile_error()
    }
}