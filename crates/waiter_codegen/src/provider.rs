use syn::{ItemStruct, ItemImpl, Type};
use proc_macro::TokenStream;

pub fn generate_component_provider_impl(component: ItemStruct) -> TokenStream {
    let comp_name = &component.ident;
    let comp_generics = &component.generics;

    let result = quote::quote! {
        impl #comp_generics Provider<#comp_name #comp_generics> for Container<profiles::Default> {
            fn get_ref(&mut self) -> &#comp_name #comp_generics {
                let type_id = std::any::TypeId::of::<#comp_name>();
                if !self.components.contains_key(&type_id) {
                    let component = Box::new(#comp_name::__waiter_create(self));
                    self.components.insert(type_id, component);
                }
                let any = self.components.get(&type_id)
                    .unwrap();

                return any
                    .downcast_ref::<#comp_name>()
                    .unwrap();
            }
        }
    };

    return TokenStream::from(result);
}

pub fn generate_interface_provider_impl(impl_block: ItemImpl) -> TokenStream {
    let (_, interface, _) = impl_block.trait_
        .expect("#[provides] can be used only on impl blocks for traits");

    let comp_name = if let Type::Path(comp_path) = *impl_block.self_ty {
        comp_path.path.segments.first().unwrap().ident.clone()
    } else {
        panic!("Failed to create provider")
    };

    let result = quote::quote! {
        impl Provider<dyn #interface> for Container<profiles::Default> {
            fn get_ref(&mut self) -> &(dyn #interface + 'static) {
                return Provider::<#comp_name>::get_ref(self);
            }
        }
    };

    return TokenStream::from(result);
}