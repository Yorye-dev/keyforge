pub mod capabilities;
pub mod discovery;
pub mod identification;
pub mod session;

use crate::domain::devices::KeyboardModel;

pub trait QmkMetadataRepository {
    fn models(&self) -> &[KeyboardModel];
    fn find(&self, id: &str) -> Option<&KeyboardModel> {
        self.models().iter().find(|m| m.id == id)
    }
}
