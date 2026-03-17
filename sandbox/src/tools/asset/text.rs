use crate::tools::asset::AssetPool;

pub struct TextAsset {
    key: Box<str>,
    parameters: Box<[TextElement]>,
}

impl TextAsset {
    pub fn simple(key: impl Into<Box<str>>) -> Self {
        Self {
            key: key.into(),
            parameters: Box::new([]),
        }
    }

    pub fn template(key: impl Into<Box<str>>, parameters: Box<[TextElement]>) -> Self {
        Self {
            key: key.into(),
            parameters,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn parameters(&self) -> &[TextElement] {
        &self.parameters
    }
}

pub enum TextElement {
    String(Box<str>),
    Asset(TextAsset),
}

impl TextElement {
    pub fn resolve_text(&self, assets: &AssetPool) -> String {
        match self {
            Self::String(string) => string.to_string(),
            Self::Asset(asset) => assets.resolve_text(asset),
        }
    }
}

impl From<&str> for TextElement {
    fn from(string: &str) -> Self {
        TextElement::String(string.into())
    }
}

impl From<Box<str>> for TextElement {
    fn from(string: Box<str>) -> Self {
        TextElement::String(string)
    }
}

impl From<String> for TextElement {
    fn from(string: String) -> Self {
        TextElement::String(string.into_boxed_str())
    }
}

impl From<TextAsset> for TextElement {
    fn from(asset: TextAsset) -> Self {
        TextElement::Asset(asset)
    }
}
