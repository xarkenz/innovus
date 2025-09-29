pub use innovus::tools::*;

pub mod asset;
pub mod input;
pub mod noise;

pub fn generate_uuid() -> Uuid {
    Uuid::new_v4()
}
