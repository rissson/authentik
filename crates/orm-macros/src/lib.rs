use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, parse::Parser, parse_macro_input, ItemStruct, DeriveInput};

#[proc_macro_attribute]
pub fn expiring_model(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let _ = parse_macro_input!(args as parse::Nothing);

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        for field in [
            quote! { pub expires: DateTimeWithTimeZone },
            quote! { pub expiring: bool },
        ] {
            fields.named.push(
                syn::Field::parse_named.parse2(field).unwrap()
            );
        }
    }

    quote! {
        #item_struct
    }.into()
}

#[proc_macro_derive(DeriveExpiringModel)]
pub fn derive_expiring_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    quote! {
        impl ExpiringModel for #name {
            fn is_expired(&self) -> bool {
                chrono::Utc::now().with_timezone(&self.expires.timezone()) > self.expires
            }
        }
    }.into()
}
