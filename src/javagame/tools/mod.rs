pub use innovus::tools::*;

pub mod input;

pub fn generate_uuid() -> Uuid {
    Uuid::new_v4()
}
