use proc_macro::TokenStream;

use syn::{GenericParam, ItemImpl, ItemStruct, Path, Type, ItemFn, ReturnType, Error};
use syn::export::{TokenStream2, ToTokens};
use std::ops::Deref;
use component::{generate_dependencies_create_code, generate_inject_dependencies_tuple, Argument};
use syn::spanned::Spanned;

pub fn generate_component_provider_impl_struct(component: ItemStruct) -> TokenStream {
    let comp_name = component.ident;
    let comp_generics = component.generics.clone();

    let create_component_code = quote::quote! {
        #comp_name::__waiter_create(self)
    };
    let inject_deferred_code = quote::quote! {
        #comp_name::__waiter_inject_deferred(self, &*component);
    };

    generate_component_provider_impl(
        quote::quote! { #comp_name #comp_generics },
        component.generics.params.iter().collect(),
        vec!(),
        create_component_code,
        inject_deferred_code
    )
}

pub fn generate_component_provider_impl_fn(profiles: Vec<&Path>, factory: ItemFn) -> TokenStream {
    let comp_name = if let ReturnType::Type(_, type_) = &factory.sig.output {
        if let Type::Path(type_path) = type_.deref() {
            type_path.path.segments.to_token_stream()
        } else {
            panic!("Unsupported return type for factory function {}", factory.sig.to_token_stream().to_string())
        }
    } else {
        panic!("Return type must be specified for factory function {}", factory.sig.to_token_stream().to_string())
    };

    let fn_name = factory.sig.ident;

    let dependencies_code = generate_dependencies_create_code(
        factory.sig.inputs.iter()
            .map(|arg| Argument::from_fn_arg(arg.clone()))
            .collect()
    );
    let factory_code = generate_inject_dependencies_tuple(factory.sig.inputs.len());

    let create_component_code = quote::quote! {
        {
            let container = &mut *self;
            #dependencies_code
            #fn_name #factory_code
        }
    };
    let inject_deferred_code = quote::quote! {};

    generate_component_provider_impl(
        comp_name,
        factory.sig.generics.params.iter()
            .filter(|p| if let GenericParam::Lifetime(_) = p { true } else { false })
            .collect(),
        profiles,
        create_component_code,
        inject_deferred_code
    )
}

pub fn generate_component_provider_impl(
    comp_name: TokenStream2,
    comp_generics: Vec<&GenericParam>,
    profiles: Vec<&Path>,
    create_component_code: TokenStream2,
    inject_deferred_code: TokenStream2
) -> TokenStream {
    let (profiles, provider_generics) = if profiles.is_empty() {
        let generic_profile = quote::quote! { PROFILE };

        let provider_generics = if comp_generics.is_empty() {
            quote::quote! { <PROFILE> }
        } else {
            quote::quote! { <#(#comp_generics),*, PROFILE> }
        };

        (vec!(generic_profile), provider_generics)
    } else {
        let profiles = profiles.iter()
            .map(|p| p.to_token_stream())
            .collect();
        (profiles, quote::quote! { <#(#comp_generics),*> })
    };

    let result = quote::quote! {#(
        impl #provider_generics waiter_di::Provider<#comp_name> for waiter_di::Container<#profiles> {
            fn get(&mut self) -> std::rc::Rc<#comp_name> {
                let type_id = std::any::TypeId::of::<#comp_name>();
                if !self.components.contains_key(&type_id) {
                    let component = std::rc::Rc::new(#create_component_code);
                    self.components.insert(type_id, component.clone());
                    #inject_deferred_code
                }
                let any = self.components.get(&type_id)
                    .unwrap();

                return any.clone()
                    .downcast::<#comp_name>()
                    .unwrap();
            }
            fn get_ref(&mut self) -> &#comp_name {
                // Value under RC is still stored in container, so it can be safely return as reference
                // that has the same life as container reference
                unsafe {
                    std::rc::Rc::as_ptr(&waiter_di::Provider::<#comp_name>::get(self))
                        .as_ref()
                        .unwrap()
                }
            }

            fn create(&mut self) -> Box<#comp_name> {
                let component = Box::new(#create_component_code);
                #inject_deferred_code
                return component;
            }
        }
    )*};

    return TokenStream::from(result);
}

pub fn generate_interface_provider_impl(profiles: Vec<&Path>, impl_block: ItemImpl) -> TokenStream {
    let interface = match impl_block.trait_ {
        Some((_, interface, _)) => interface,
        None => return TokenStream::from(Error::new(
            impl_block.span(),
            "#[provides] can be used only on impl blocks for traits"
        ).to_compile_error())
    };

    let comp_name = if let Type::Path(comp_path) = *impl_block.self_ty {
        comp_path.path.segments.first().unwrap().ident.clone()
    } else {
        return TokenStream::from(Error::new(impl_block.self_ty.span(), "Failed to create provider").to_compile_error())
    };

    let provider_body = quote::quote! {{
        fn get(&mut self) -> std::rc::Rc<dyn #interface> {
            waiter_di::Provider::<#comp_name>::get(self)
        }
        fn get_ref(&mut self) -> &(dyn #interface + 'static) {
            waiter_di::Provider::<#comp_name>::get_ref(self)
        }
        fn create(&mut self) -> Box<dyn #interface> {
            waiter_di::Provider::<#comp_name>::create(self)
        }
    }};

    let result = if profiles.is_empty() {
        quote::quote! {
            impl<P> waiter_di::Provider<dyn #interface> for waiter_di::Container<P> #provider_body
        }
    } else {
        quote::quote! {
            #(impl waiter_di::Provider<dyn #interface> for waiter_di::Container<#profiles> #provider_body)*
        }
    };

    return TokenStream::from(result);
}