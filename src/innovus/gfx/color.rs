use std::io::{BufRead, BufReader, Read};
use crate::tools::Vector;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct RGBColor(pub Vector<f32, 3>);

impl RGBColor {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self(Vector([r, g, b]))
    }

    pub const fn black() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub const fn white() -> Self {
        Self::new(1.0, 1.0, 1.0)
    }

    pub const fn r(&self) -> f32 {
        self.0.x()
    }

    pub const fn g(&self) -> f32 {
        self.0.y()
    }

    pub const fn b(&self) -> f32 {
        self.0.z()
    }
}

#[derive(Clone, Debug)]
pub struct ColorPalette {
    name: String,
    colors: Vec<RGBColor>,
}

impl ColorPalette {
    pub fn new(name: String, colors: Vec<RGBColor>) -> Self {
        Self { name, colors }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn colors(&self) -> &[RGBColor] {
        &self.colors
    }

    pub fn parse_gpl(reader: impl Read) -> Result<Self, String> {
        let mut name = String::new();
        let mut colors = Vec::new();

        let mut lines = BufReader::new(reader).lines();
        let first_line = lines.next().unwrap_or(Ok(String::new()));
        let first_line = first_line.map_err(|err| err.to_string())?;
        if first_line != "GIMP Palette" {
            return Err("first line of palette must be 'GIMP Palette'".into());
        }

        for line in lines {
            let line = line.map_err(|err| err.to_string())?;
            if let Some(parsed_name) = line.strip_prefix("Name: ") {
                name = parsed_name.into();
                continue;
            }
            let mut split = line.split_whitespace();
            let (Some(r), Some(g), Some(b)) = (split.next(), split.next(), split.next()) else {
                continue;
            };
            let (Ok(r), Ok(g), Ok(b)) = (r.parse::<u8>(), g.parse::<u8>(), b.parse::<u8>()) else {
                continue;
            };
            colors.push(RGBColor::new(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
            ));
        }

        Ok(Self::new(name, colors))
    }
}
