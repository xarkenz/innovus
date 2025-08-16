use json::JsonValue;
use innovus::tools::{Rectangle, Vector};
use crate::tools::asset::ImageAnimation;

#[derive(Clone, PartialEq, Debug)]
pub struct EntityImage {
    pub world_offset: Rectangle<f32>,
    pub atlas_base_region: Rectangle<u32>,
    pub animation: Option<ImageAnimation>,
}

impl EntityImage {
    pub fn parse(key: &str, atlas_region: Rectangle<u32>, metadata: &JsonValue) -> Result<Self, String> {
        let mut missing_keys = Vec::new();

        let x = metadata["x"].as_f32().unwrap_or_else(|| {
            missing_keys.push("x");
            Default::default()
        });
        let y = metadata["y"].as_f32().unwrap_or_else(|| {
            missing_keys.push("y");
            Default::default()
        });
        let width = metadata["width"].as_f32().unwrap_or_else(|| {
            missing_keys.push("width");
            Default::default()
        });
        let height = metadata["height"].as_f32().unwrap_or_else(|| {
            missing_keys.push("height");
            Default::default()
        });

        if !missing_keys.is_empty() {
            let mut err_string = format!("missing or invalid keys for entity/{key}: {}", missing_keys[0]);
            for &missing_key in &missing_keys[1..] {
                err_string.push_str(", ");
                err_string.push_str(missing_key);
            }
            return Err(err_string);
        }

        let mut atlas_base_region = atlas_region;
        let animation;
        if let Some(frame_count) = metadata["animation"]["frame_count"].as_u32() {
            let Some(frame_time) = metadata["animation"]["frame_time"].as_u32() else {
                return Err(format!("missing key for entity/{key}: animation.frame_time"));
            };
            if frame_count == 0 || atlas_region.height() % frame_count != 0 {
                return Err(format!("invalid frame count for entity/{key}: image height ({}) not divisible by animation.frame_count ({frame_count})", atlas_region.height()));
            }
            atlas_base_region.set_max_y(atlas_region.min_y() + atlas_region.height() / frame_count);
            animation = Some(ImageAnimation {
                frame_count,
                frame_time,
            });
        }
        else {
            animation = None;
        }

        const UNITS_PER_PIXEL: f32 = 0.0625; // 1/16
        Ok(Self {
            world_offset: Rectangle::from_size(
                Vector([x, y]) * UNITS_PER_PIXEL,
                Vector([width, height]) * UNITS_PER_PIXEL,
            ),
            atlas_base_region,
            animation,
        })
    }
}
