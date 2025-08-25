use json::JsonValue;
use innovus::tools::{Rectangle, Vector};
use crate::tools::asset::anim::ImageAnimation;

#[derive(Clone, PartialEq, Debug)]
pub struct EntityImage {
    pub world_offset: Rectangle<f32>,
    pub atlas_base_region: Rectangle<u32>,
    pub animation: Option<ImageAnimation>,
}

impl EntityImage {
    pub fn parse(key: &str, atlas_region: Rectangle<u32>, metadata: &JsonValue) -> Result<Self, String> {
        let mut missing_properties = Vec::new();

        let x = metadata["x"].as_f32().unwrap_or_else(|| {
            missing_properties.push("x");
            Default::default()
        });
        let y = metadata["y"].as_f32().unwrap_or_else(|| {
            missing_properties.push("y");
            Default::default()
        });
        let width = metadata["width"].as_f32().unwrap_or_else(|| {
            missing_properties.push("width");
            Default::default()
        });
        let height = metadata["height"].as_f32().unwrap_or_else(|| {
            missing_properties.push("height");
            Default::default()
        });

        if !missing_properties.is_empty() {
            let mut err_string = format!("missing or invalid properties for {key}: {}", missing_properties[0]);
            for &missing_property in &missing_properties[1..] {
                err_string.push_str(", ");
                err_string.push_str(missing_property);
            }
            return Err(err_string);
        }

        let animation = ImageAnimation::try_parse(key, &metadata["animation"])?;
        let mut atlas_base_region = atlas_region;
        if let Some(ImageAnimation { frame_count, .. }) = animation {
            if frame_count == 0 || atlas_region.height() % frame_count != 0 {
                return Err(format!("invalid frame count for {key}: image height ({}) not divisible by animation.frame_count ({frame_count})", atlas_region.height()));
            }
            atlas_base_region.set_max_y(atlas_region.min_y() + atlas_region.height() / frame_count);
        }

        Ok(Self {
            world_offset: Rectangle::from_size(
                Vector([x, y]) / 16.0,
                Vector([width, height]) / 16.0,
            ),
            atlas_base_region,
            animation,
        })
    }
}
