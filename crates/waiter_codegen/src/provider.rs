
//pub fn generate_component_provider_impl(component: ItemStruct) -> TokenStream {
//    let comp_name = &component.ident;
//    let comp_generics = &component.generics;
////
////    let result = quote::quote! {
////        impl #comp_generics waiter::Component for #comp_name #comp_generics {
////            fn __waiter_create() -> Self {
////                #dependencies_code
////                return #comp_name #factory_code;
////            }
////        }
////    };
////
////    return TokenStream::from(result);
//    return component;
//}