use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct Db(pub DatabaseConnection);
