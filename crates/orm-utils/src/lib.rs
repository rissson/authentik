pub mod expiring_model;

pub mod prelude {
    pub use authentik_orm_macros::expiring_model;
    pub use authentik_orm_macros::DeriveExpiringModel;
    pub use crate::expiring_model::ExpiringModel;
    pub use authentik_orm_macros::DeriveExpiringModelAction;
    pub use crate::expiring_model::ExpiringModelAction;
}
