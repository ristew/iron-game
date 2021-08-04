extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::{ quote, format_ident };
use syn::{ parse_macro_input, DeriveInput };

#[proc_macro_attribute]
pub fn iron_data(attr: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as DeriveInput);
    let name = &parsed_input.ident;
    let name_id = format_ident!("{}Id", name);
    let name_ptr = format_ident!("{}Ptr", name);
    let name_id_str = format!("{}", name_id);

    let expanded = quote! {
        #[derive(Clone)]
        pub struct #name_id(usize, RefCell<Option<Weak<RefCell<#name>>>>);

        impl PartialEq for #name_id {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl Eq for #name_id {}

        impl std::fmt::Debug for #name_id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(format!("{}({})", #name_id_str, self.0).as_str())
            }
        }

        impl std::hash::Hash for #name_id {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }

        pub type #name_ptr = std::rc::Rc<std::cell::RefCell<#name>>;

        impl IronId for #name_id {
            type Target = #name;

            fn try_borrow(&self) -> Option<#name_ptr> {
                self.1.borrow().as_ref().map(|weak| { weak.upgrade().unwrap().clone() })
            }

            fn set_reference(&self, reference: std::rc::Rc<std::cell::RefCell<Self::Target>>) {
                *self.1.borrow_mut() = Some(std::rc::Rc::downgrade(&reference));
            }

            fn new(id: usize) -> Self {
                Self(id, std::cell::RefCell::new(None))
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

            fn id(&self) -> Self::IdType {
                self.id.clone()
            }
        }
    };

    TokenStream::from(expanded)
}
