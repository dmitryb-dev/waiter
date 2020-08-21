use proc_macro::TokenStream;

use syn::{GenericParam, ItemImpl, ItemStruct, Path, Type};

pub fn generate_component_provider_impl(component: ItemStruct) -> TokenStream {
    let comp_name = &component.ident;

    let comp_generics = component.generics;
    let provider_generics = if comp_generics.params.is_empty() {
        quote::quote! { <PROFILE> }
    } else {
        let existed_generics: Vec<&GenericParam> = comp_generics.params.iter().collect();
        quote::quote! { <#(#existed_generics),*, PROFILE> }
    };

    let result = quote::quote! {
        impl #provider_generics waiter::Provider<#comp_name #comp_generics> for Container<PROFILE> {
            fn get(&mut self) -> std::rc::Rc<#comp_name #comp_generics> {
                let type_id = std::any::TypeId::of::<#comp_name>();
                if !self.components.contains_key(&type_id) {
                    let component = std::rc::Rc::new(#comp_name::__waiter_create(self));
                    #comp_name::__waiter_inject_deferred(self, component.clone());
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
                Box::new(#comp_name::__waiter_create(self))
            }
        }
    };

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
            waiter::Provider::<#comp_name>::get(self)
        }
        fn get_ref(&mut self) -> &(dyn #interface + 'static) {
            waiter::Provider::<#comp_name>::get_ref(self)
        }
        fn create(&mut self) -> Box<dyn #interface> {
            waiter::Provider::<#comp_name>::create(self)
        }
    }};

    let result = if profiles.is_empty() {
        quote::quote! {
            impl<P> waiter::Provider<dyn #interface> for Container<P> #provider_body
        }
    } else {
        quote::quote! {
            #(impl waiter::Provider<dyn #interface> for Container<#profiles> #provider_body)*
        }
    };

    return TokenStream::from(result);
}