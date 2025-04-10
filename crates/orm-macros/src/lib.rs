use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, ItemStruct, parse, parse::Parser, parse_macro_input};

#[proc_macro_attribute]
pub fn expiring_model(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let _ = parse_macro_input!(args as parse::Nothing);

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        for field in [
            quote! { pub expires: DateTimeWithTimeZone },
            quote! { pub expiring: bool },
        ] {
            fields.named.push(syn::Field::parse_named.parse2(field).unwrap());
        }
    }

    quote! {
        #item_struct
    }
    .into()
}

#[proc_macro_derive(ExpiringModel)]
pub fn derive_expiring_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    quote! {
        impl crate::orm::utils::expiring_model::ExpiringModel for #name {
            fn is_expired(&self) -> bool {
                chrono::Utc::now().with_timezone(&self.expires.timezone()) > self.expires
            }
        }
    }
    .into()
}

#[proc_macro_derive(ExpiringModelAction)]
pub fn derive_expiring_model_action(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    quote! {
        #[async_trait::async_trait]
        impl crate::orm::utils::expiring_model::ExpiringModelAction for #name {
            type Entity = Entity;

            async fn expire_action<'a, A, C>(self, db: &'a C) -> Result<sea_orm::DeleteResult, sea_orm::DbErr>
            where
                Self: sea_orm::IntoActiveModel<A> + crate::orm::utils::expiring_model::ExpiringModel,
                C: sea_orm::ConnectionTrait,
                A: sea_orm::ActiveModelTrait<Entity = Self::Entity> + sea_orm::ActiveModelBehavior + Send + 'a,
            {
                self.delete(db).await
            }
        }
    }
    .into()
}
