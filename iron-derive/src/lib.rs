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

    let expanded = quote! {
        #[derive(Clone)]
        pub struct #name_id(usize, RefCell<Option<Weak<RefCell<#name>>>>);

        impl PartialEq for #name_id {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl Eq for #name_id {}

        impl Debug for #name_id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(format!("#name_id({})", self.0).as_str())
            }
        }

        impl Hash for #name_id {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }

        impl IronId for #name_id {
            type Target = #name;

            fn try_borrow(&self) -> Option<Rc<RefCell<Self::Target>>> {
                self.1.borrow().as_ref().map(|weak| { weak.upgrade().unwrap().clone() })
            }

            fn set_reference(&self, reference: Rc<RefCell<Self::Target>>) {
                *self.1.borrow_mut() = Some(Rc::downgrade(&reference));
            }

            fn new(id: usize) -> Self {
                Self(id, RefCell::new(None))
            }
        }

        #[derive(IronData)]
        #parsed_input

        pub type #name_ptr = Rc<RefCell<#name>>;
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
        impl #impl_generics crate::game::IronData<#name_id> for #name #ty_generics #where_clause {
            type DataType = #name;

            fn id(&self) -> #name_id {
                self.id.clone()
            }
        }
    };

    TokenStream::from(expanded)
}
