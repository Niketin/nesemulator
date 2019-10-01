

pub struct Display {
    width: usize,
    height: usize,
    array: Vec<Color>
}

impl Display {
    pub fn new(width: usize, height: usize) -> Display {
        Display {
            width,
            height,
            array: vec![Color::new(); width * height]
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> &Color {
        &self.array[self.width * y + x]
    }

    pub fn get_pixels(&self) -> &Vec<Color> {
        &self.array
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        self.array[self.width * y + x] = color;
    }
}

impl Default for Display {
    fn default() -> Self { 
        Self::new(256, 240)
    }
}

#[derive(Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Color {
    fn new() -> Color {
        Color { r: 0, g: 0, b: 0}
    }

    fn new_rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }
}

