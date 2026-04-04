use std::io::{BufRead, BufReader, Read};
use crate::tools::Vector;

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub enum Color {
    #[default]
    Black,
    White,
    Red,
    Green,
    Blue,
    Cyan,
    Magenta,
    Yellow,
    Value(f32),
    IntRGB(Vector<u8, 3>),
    RGB(Vector<f32, 3>),
}

impl Color {
    pub fn rgb(&self) -> Vector<f32, 3> {
        match *self {
            Self::Black => Vector::zero(),
            Self::White => Vector::one(),
            Self::Red => Vector([1.0, 0.0, 0.0]),
            Self::Green => Vector([0.0, 1.0, 0.0]),
            Self::Blue => Vector([0.0, 0.0, 1.0]),
            Self::Cyan => Vector([0.0, 1.0, 1.0]),
            Self::Magenta => Vector([1.0, 0.0, 1.0]),
            Self::Yellow => Vector([1.0, 1.0, 0.0]),
            Self::Value(value) => Vector::splat(value),
            Self::IntRGB(rgb) => rgb.map(|x| x as f32 / 255.0),
            Self::RGB(rgb) => rgb,
        }
    }

    pub fn with_alpha(self, alpha: f32) -> AlphaColor {
        AlphaColor::new(self, alpha)
    }
}

impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self {
        let Vector([r, g, b]) = color.rgb().map(f64::from);
        Self {
            r,
            g,
            b,
            a: 1.0,
        }
    }
}

impl std::ops::Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self::RGB(self.rgb() * rhs.rgb())
    }
}

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct AlphaColor {
    pub color: Color,
    pub alpha: f32,
}

impl AlphaColor {
    pub fn new(color: Color, alpha: f32) -> Self {
        Self {
            color,
            alpha,
        }
    }

    pub fn opaque(color: Color) -> Self {
        Self::new(color, 1.0)
    }

    pub fn rgb(&self) -> Vector<f32, 3> {
        self.color.rgb()
    }

    pub fn rgba(&self) -> Vector<f32, 4> {
        self.color.rgb().with_w(self.alpha)
    }
}

impl From<Color> for AlphaColor {
    fn from(color: Color) -> Self {
        Self::opaque(color)
    }
}

impl From<Vector<f32, 4>> for AlphaColor {
    fn from(rgba: Vector<f32, 4>) -> Self {
        Self::new(Color::RGB(rgba.xyz()), rgba.w())
    }
}

impl From<AlphaColor> for wgpu::Color {
    fn from(color: AlphaColor) -> Self {
        let Vector([r, g, b]) = color.color.rgb().map(f64::from);
        Self {
            r,
            g,
            b,
            a: color.alpha.into(),
        }
    }
}

impl std::ops::Mul for AlphaColor {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        (self.rgba() * rhs.rgba()).into()
    }
}

#[derive(Clone, Debug)]
pub struct ColorPalette {
    name: String,
    colors: Vec<Color>,
}

impl ColorPalette {
    pub fn new(name: String, colors: Vec<Color>) -> Self {
        Self {
            name,
            colors,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn colors(&self) -> &[Color] {
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
            colors.push(Color::IntRGB(Vector([r, g, b])));
        }

        Ok(Self::new(name, colors))
    }
}
