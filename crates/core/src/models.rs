pub mod provider {
    use sea_orm::entity::prelude::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "authentik_core_provider")]
    pub struct Model {
        #[sea_orm(primary_key)]
        #[serde(skip_deserializing)]
        pub id: i32,
        #[sea_orm(column_type = "Text")]
        pub name: String,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub use provider::Entity as Provider;
