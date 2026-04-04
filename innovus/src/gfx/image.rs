use std::path::Path;
use crate::tools::Vector;

// TODO: make generic over data per pixel
pub struct Image {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl Image {
    pub fn empty() -> Self {
        Self {
            data: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    pub fn new(width: u32, height: u32) -> Self {
        let mut data = Vec::with_capacity(width as usize * height as usize * 4);
        data.resize(data.capacity(), 0);
        Self {
            data,
            width,
            height,
        }
    }

    pub fn from_dynamic_image(image: &image::DynamicImage) -> Self {
        Self {
            data: image.to_rgba8().to_vec(),
            width: image.width(),
            height: image.height(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, image::ImageError> {
        Ok(Self::from_dynamic_image(&image::load_from_memory(bytes)?))
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ImageFromFileError> {
        Ok(Self::from_dynamic_image(&image::ImageReader::open(path)
            .map_err(ImageFromFileError::Open)?
            .decode()
            .map_err(ImageFromFileError::Decode)?))
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size(&self) -> Vector<u32, 2> {
        Vector([self.width, self.height])
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

#[derive(Debug)]
pub enum ImageFromFileError {
    Open(std::io::Error),
    Decode(image::ImageError),
}

impl std::fmt::Display for ImageFromFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open(error) => error.fmt(f),
            Self::Decode(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for ImageFromFileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Open(error) => Some(error),
            Self::Decode(error) => Some(error),
        }
    }
}

impl Default for Image {
    fn default() -> Self {
        Self::empty()
    }
}

pub struct ImageAtlas {
    image: Image,
    flow: ImageAtlasFlow,
    flow_x: u32,
    flow_y: u32,
}

#[derive(Copy, Clone, Debug, Default)]
pub enum ImageAtlasFlow {
    LeftToRight,
    #[default] // Most efficient to construct
    TopToBottom,
}

impl ImageAtlas {
    pub fn new(flow: ImageAtlasFlow) -> Self {
        Self {
            image: Image::empty(),
            flow,
            flow_y: 0,
            flow_x: 0,
        }
    }

    pub fn image(&self) -> &Image {
        &self.image
    }

    pub fn image_mut(&mut self) -> &mut Image {
        &mut self.image
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }

    pub fn size(&self) -> Vector<u32, 2> {
        self.image.size()
    }

    pub fn data(&self) -> &[u8] {
        self.image.data()
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        self.image.data_mut()
    }

    pub fn add_image(&mut self, image: &Image) -> Vector<u32, 2> {
        let mut flow_x = self.flow_x;
        let mut flow_y = self.flow_y;

        // Check bounds and calculate the new flow point
        match self.flow {
            ImageAtlasFlow::LeftToRight => {
                self.flow_x = flow_x.checked_add(image.width()).unwrap_or_else(|| {
                    // Overflow into a new row
                    self.flow_y = self.height();
                    flow_y = self.flow_y;
                    flow_x = 0;
                    image.width()
                })
            }
            ImageAtlasFlow::TopToBottom => {
                self.flow_y = flow_y.checked_add(image.height()).unwrap_or_else(|| {
                    // Overflow into a new column
                    self.flow_x = self.width();
                    flow_x = self.flow_x;
                    flow_y = 0;
                    image.height()
                })
            }
        }

        if flow_x + image.width() > self.width() {
            // Expand the atlas width (and maybe height). Might as well just make a new array, lol
            let expanded_width = flow_x + image.width();
            let expanded_height = self.height().max(flow_y + image.height());
            let mut expanded_data = Vec::with_capacity(expanded_width as usize * expanded_height as usize * 4);
            expanded_data.resize(expanded_data.capacity(), 0);
            // Copy from old data to new data, row by row
            for y in 0 .. self.height() as usize {
                let src_start = y * self.width() as usize * 4;
                let dst_start = y * expanded_width as usize * 4;
                let length = self.width() as usize * 4;
                expanded_data[dst_start .. dst_start + length]
                    .copy_from_slice(&self.data()[src_start .. src_start + length]);
            }
            // Replace the old image data
            self.image.data = expanded_data;
            self.image.width = expanded_width;
            self.image.height = expanded_height;
        }
        else if flow_y + image.height() > self.height() {
            // Only expand the atlas in the Y direction. No need for mass copying in this case
            self.image.height = flow_y + image.height();
            self.image.data.resize(self.width() as usize * self.height() as usize * 4, 0);
        }

        // Write the new image into place, row by row
        for y_offset in 0 .. image.height() as usize {
            let src_start = y_offset * image.width() as usize * 4;
            let dst_start = ((flow_y as usize + y_offset) * self.width() as usize * 4) + flow_x as usize * 4;
            let length = image.width() as usize * 4;
            self.data_mut()[dst_start .. dst_start + length]
                .copy_from_slice(&image.data()[src_start .. src_start + length]);
        }

        Vector([flow_x, flow_y])
    }

    pub fn next_flow(&mut self) {
        match self.flow {
            ImageAtlasFlow::LeftToRight => {
                self.flow_y = self.height();
                self.flow_x = 0;
            }
            ImageAtlasFlow::TopToBottom => {
                self.flow_x = self.width();
                self.flow_y = 0;
            }
        }
    }

    pub fn clear(&mut self) {
        self.image.data.clear();
        self.image.width = 0;
        self.image.height = 0;
        self.flow_x = 0;
        self.flow_y = 0;
    }
}

impl Default for ImageAtlas {
    fn default() -> Self {
        Self::new(ImageAtlasFlow::default())
    }
}
