pub mod oauth2_provider {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "authentik_providers_oauth2_oauth2provider")]
    pub struct Model {
        #[sea_orm(primary_key)]
        #[serde(skip_deserializing)]
        pub provider_ptr_id: i32,
        #[sea_orm(column_type = "String(StringLen::N(255))")]
        pub client_id: String,
        #[sea_orm(column_type = "String(StringLen::N(255))")]
        pub client_secret: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "crate::core::models::provider::Entity",
            from = "Column::ProviderPtrId",
            to = "crate::core::models::provider::Column::Id"
        )]
        Provider,
    }

    impl Related<crate::core::models::provider::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::Provider.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}
pub use oauth2_provider::Entity as OAuth2Provider;

pub mod access_token {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    use crate::{orm::utils::prelude::*, providers::oauth2::id_token::IDToken};
    #[expiring_model]
    #[derive(
        Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, ExpiringModel, ExpiringModelAction,
    )]
    #[sea_orm(table_name = "authentik_providers_oauth2_accesstoken")]
    pub struct Model {
        #[sea_orm(primary_key)]
        #[serde(skip_deserializing)]
        pub id: i32,
        pub provider_id: i32,
        pub revoked: bool,
        #[sea_orm(column_type = "Text")]
        pub _scope: String,

        #[sea_orm(column_type = "Text")]
        pub token: String,
        #[sea_orm(column_type = "Text")]
        pub _id_token: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::oauth2_provider::Entity",
            from = "Column::ProviderId",
            to = "super::oauth2_provider::Column::ProviderPtrId"
        )]
        OAuth2Provider,
    }

    impl Related<super::oauth2_provider::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::OAuth2Provider.def()
        }
    }

    impl Model {
        pub(crate) fn get_scope(&self) -> Vec<&str> {
            self._scope.split(' ').collect()
        }

        pub(crate) fn get_id_token(&self) -> serde_json::Result<IDToken> {
            Ok(serde_json::from_str(&self._id_token)?)
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}
pub use access_token::Entity as AccessToken;

pub mod refresh_token {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    use crate::{orm::utils::prelude::*, providers::oauth2::id_token::IDToken};
    #[expiring_model]
    #[derive(
        Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, ExpiringModel, ExpiringModelAction,
    )]
    #[sea_orm(table_name = "authentik_providers_oauth2_refreshtoken")]
    pub struct Model {
        #[sea_orm(primary_key)]
        #[serde(skip_deserializing)]
        pub id: i32,
        pub provider_id: i32,
        pub revoked: bool,
        #[sea_orm(column_type = "Text")]
        pub _scope: String,

        #[sea_orm(column_type = "Text")]
        pub token: String,
        #[sea_orm(column_type = "Text")]
        pub _id_token: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::oauth2_provider::Entity",
            from = "Column::ProviderId",
            to = "super::oauth2_provider::Column::ProviderPtrId"
        )]
        OAuth2Provider,
    }

    impl Related<super::oauth2_provider::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::OAuth2Provider.def()
        }
    }

    impl Model {
        pub(crate) fn get_scope(&self) -> Vec<&str> {
            self._scope.split(' ').collect()
        }

        pub(crate) fn get_id_token(&self) -> serde_json::Result<IDToken> {
            Ok(serde_json::from_str(&self._id_token)?)
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}
pub use refresh_token::Entity as RefreshToken;
