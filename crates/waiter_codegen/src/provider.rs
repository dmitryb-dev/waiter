use syn::ItemStruct;
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