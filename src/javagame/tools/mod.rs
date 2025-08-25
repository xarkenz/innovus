pub use innovus::tools::*;

pub mod input;
pub mod asset;
pub mod noise;

pub fn generate_uuid() -> Uuid {
    Uuid::new_v4()
}
