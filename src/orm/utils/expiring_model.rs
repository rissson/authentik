use async_trait::async_trait;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ConnectionTrait, DbErr, DeleteResult, EntityTrait, IntoActiveModel,
};

/// Base trait for models that can expire and are automatically cleanup up
pub trait ExpiringModel {
    fn is_expired(&self) -> bool;
}

#[async_trait]
pub trait ExpiringModelAction {
    type Entity: EntityTrait;

    async fn expire_action<'a, A, C>(self, db: &'a C) -> Result<DeleteResult, DbErr>
    where
        Self: IntoActiveModel<A> + ExpiringModel,
        C: ConnectionTrait,
        A: ActiveModelTrait<Entity = Self::Entity> + ActiveModelBehavior + Send + 'a;
}
