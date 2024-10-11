pub trait ExpiringModel {
    fn is_expired(&self) -> bool;
}
