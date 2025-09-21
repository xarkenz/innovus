use json::JsonValue;

#[derive(Clone, PartialEq, Debug)]
pub struct ImageAnimation {
    pub frame_count: u32,
    pub frame_time: u32,
}

impl ImageAnimation {
    pub fn try_parse(key: &str, metadata: &JsonValue) -> Result<Option<Self>, String> {
        if let Some(frame_count) = metadata["frame_count"].as_u32() {
            let Some(frame_time) = metadata["frame_time"].as_u32() else {
                return Err(format!("missing property for {key}: animation.frame_time"));
            };
            Ok(Some(ImageAnimation {
                frame_count,
                frame_time,
            }))
        }
        else {
            Ok(None)
        }
    }
}
