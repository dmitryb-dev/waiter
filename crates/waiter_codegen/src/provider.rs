use proc_macro::TokenStream;

use syn::{GenericParam, ItemImpl, ItemStruct, Path, Type, ItemFn, ReturnType};
use syn::export::{TokenStream2, ToTokens};
use std::ops::Deref;

pub fn generate_component_provider_impl_struct(component: ItemStruct) -> TokenStream {
    let comp_name = component.ident;

    let create_component_code = quote::quote! {
        Box::new(#comp_name::__waiter_create(self));
        #comp_name::__waiter_inject_deferred(self, &component)
    };

    generate_component_provider_impl(
        comp_name.to_token_stream(),
        component.generics.params.iter().collect(),
        vec!(),
        create_component_code
    )
}

pub fn generate_component_provider_impl_fn(profiles: Vec<&Path>, factory: ItemFn) -> TokenStream {
    let comp_name = if let ReturnType::Type(_, type_) = &factory.sig.output {
        if let Type::Path(type_path) = type_.deref() {
            type_path.path.segments.first().unwrap().ident.to_token_stream()
        } else {
            panic!("Unsupported return type for factory function {}", factory.sig.to_token_stream().to_string())
        }
    } else {
        panic!("Return type must be specified for factory function {}", factory.sig.to_token_stream().to_string())
    };

    let fn_name = factory.sig.ident;

    let create_component_code = quote::quote! {
        Box::new(#fn_name(self))
    };

    generate_component_provider_impl(
        comp_name,
        factory.sig.generics.params.iter()
            .filter(|p| if let GenericParam::Lifetime(_) = p { true } else { false })
            .collect(),
        profiles,
        create_component_code
    )
}

pub fn generate_component_provider_impl(
    comp_name: TokenStream2,
    comp_generics: Vec<&GenericParam>,
    profiles: Vec<&Path>,
    create_component_code: TokenStream2
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

    let comp_generics = quote::quote! { <#(#comp_generics),*> };

    let result = quote::quote! {#(
        impl #provider_generics waiter_di::Provider<#comp_name #comp_generics> for Container<#profiles> {
            fn get(&mut self) -> std::rc::Rc<#comp_name #comp_generics> {
                let type_id = std::any::TypeId::of::<#comp_name>();
                if !self.components.contains_key(&type_id) {
                    let component = Rc::<#comp_name>::from(Provider::<#comp_name>::create(self));
                    self.components.insert(type_id, component);
                }
                let any = self.components.get(&type_id)
                    .unwrap();

                return any.clone()
                    .downcast::<#comp_name>()
                    .unwrap();
            }
            fn get_ref(&mut self) -> &#comp_name #comp_generics {
                  unsafe {
                    std::rc::Rc::as_ptr(&Provider::<#comp_name>::get(self))
                        .as_ref()
                        .unwrap()
                }
            }

            fn create(&mut self) -> Box<#comp_name #comp_generics> {
                let mut component = #create_component_code;
                return component;
            }
        }
    )*};

    return TokenStream::from(result);
}

pub fn generate_interface_provider_impl(profiles: Vec<&Path>, impl_block: ItemImpl) -> TokenStream {
    let (_, interface, _) = impl_block.trait_
        .expect("#[provides] can be used only on impl blocks for traits");

    let comp_name = if let Type::Path(comp_path) = *impl_block.self_ty {
        comp_path.path.segments.first().unwrap().ident.clone()
    } else {
        panic!("Failed to create provider")
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
            impl<P> waiter_di::Provider<dyn #interface> for Container<P> #provider_body
        }
    } else {
        quote::quote! {
            #(impl waiter_di::Provider<dyn #interface> for Container<#profiles> #provider_body)*
        }
    };

    return TokenStream::from(result);
}