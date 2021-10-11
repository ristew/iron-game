extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_attribute]
pub fn iron_data(attr: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as DeriveInput);
    let name = &parsed_input.ident;
    let name_id = format_ident!("{}Id", name);
    let name_ref = format_ident!("{}Ref", name);
    let name_ptr = format_ident!("{}Ptr", name);
    let name_id_str = format!("{}", name_id);

    let expanded = quote! {
        #[derive(IronId, Copy, Clone, Serialize, Deserialize)]
        pub struct #name_id {
            num: usize,
        }

        #[derive(IronRef, Clone)]
        pub struct #name_ref {
            id: #name_id,
            inner: IronIdInner<#name>,
        }

        impl From<#name_ref> for #name_id {
            fn from(sr: SettlementRef) -> Self {
                sr.id
            }
        }

        impl PartialEq for #name_id {
            fn eq(&self, other: &Self) -> bool {
                self.num == other.num
            }
        }

        impl Eq for #name_id {}

        impl std::hash::Hash for #name_id {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.num.hash(state);
            }
        }

        impl std::fmt::Debug for #name_id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(format!("{}({})", #name_id_str, self.num).as_str())
            }
        }

        pub type #name_ptr = std::rc::Rc<std::cell::RefCell<#name>>;

        impl #name_id {
            pub fn get<'a>(&'a self) -> std::cell::Ref<'a, #name> {
                self.get_inner().borrow()
            }

            pub fn get_mut<'a>(&'a self) -> impl std::ops::DerefMut<Target = <Self as IronId>::Target> + 'a {
                self.get_inner().borrow_mut()
            }
        }

        // impl<T> Debug for T where T: IronId {
        //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //         f.write_str(format!("#name_id({})", self.0).as_str())
        //     }
        // }

        #[derive(IronData)]
        #parsed_input
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(IronId)]
pub fn iron_id_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let target = format_ident!("{}", name.to_string().replace("Id", ""));
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics crate::game::IronId for #name #ty_generics #where_clause {
            type Target = #target;

            fn new(num: usize) -> Self {
                Self {
                    num,
                }
            }

            fn num(&self) -> usize {
                self.num
            }
            fn gid(&self) -> GameId {
                GameId::#target(self.num())
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(IronRef)]
pub fn iron_ref_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let target = format_ident!("{}", name.to_string().replace("Ref", ""));
    let id = format_ident!("{}", name.to_string().replace("Ref", "Id"));
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics crate::game::IronRef for #name #ty_generics #where_clause {
            type Target = #target;
            type Id = #id;

            fn new(num: usize, inner: IronIdInner<Self::Target>) -> Self {
                Self {
                    num,
                    inner: Some(inner),
                }
            }

            fn get_inner<'a>(&'a self) -> &'a IronIdInner<Self::Target> {
                self.inner.as_ref().unwrap()
            }
        }
    };

    TokenStream::from(expanded)
}


#[proc_macro_derive(IronData)]
pub fn iron_data_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let name_id = format_ident!("{}Id", name);
    let generics = input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics crate::game::IronData for #name #ty_generics #where_clause {
            type DataType = #name;
            type IdType = #name_id;
            // type StorageType = crate::storage::Storage<Object = #name>;

            fn id(&self, world: &World) -> Self::IdType {
                world.storages.get_storage::<Self::IdType>().get_id(self.id)
            }

            fn set_id(&mut self, id: usize) {
                self.id = id;
            }
        }
    };

    TokenStream::from(expanded)
}
