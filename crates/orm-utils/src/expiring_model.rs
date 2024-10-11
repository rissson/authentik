use async_trait::async_trait;
use sea_orm::{DeleteResult, DbErr, ActiveModelTrait, ActiveModelBehavior, IntoActiveModel, ConnectionTrait, EntityTrait};

pub trait ExpiringModel {
    fn is_expired(&self) -> bool;
}

#[async_trait]
pub trait ExpiringModelAction {
    type Entity: EntityTrait;

    async fn expire_action<'a, A, C>(self, db: &'a C) -> Result<DeleteResult, DbErr>
    where
        Self: IntoActiveModel<A>,
        C: ConnectionTrait,
        A: ActiveModelTrait<Entity = Self::Entity> + ActiveModelBehavior + Send + 'a,
    ;
}
