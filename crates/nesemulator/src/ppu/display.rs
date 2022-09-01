

pub struct Display {
    pub width: usize,
    pub height: usize,
    array: Vec<u8>
}

impl Display {
    pub fn new(width: usize, height: usize) -> Display {
        Display {
            width,
            height,
            array: vec![0u8; width * height * 3]
        }
    }
    

    pub fn get_pixel(&self, x: usize, y: usize) -> &[u8] {
        &self.array[(self.width * y + x) * 3 .. (self.width * y + x) * 3 + 2]
    }

    pub fn get_pixels(&self) -> &[u8] {
        &self.array[..]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        self.array[(self.width * y + x) * 3 ] = color.r;
        self.array[(self.width * y + x) * 3 + 1] = color.g;
        self.array[(self.width * y + x) * 3 + 2] = color.b;
    }
}

impl Default for Display {
    fn default() -> Self { 
        Self::new(256, 240)
    }
}

#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Color {
    pub const fn new() -> Color {
        Color { r: 0, g: 0, b: 0}
    }

    pub const fn new_rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }
}

impl IntoIterator for Color {
    type Item = u8;
    type IntoIter = ColorIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        ColorIntoIterator {
            color: self,
            index: 0,
        }
    }
}

pub struct ColorIntoIterator {
    color: Color,
    index: usize,
}

impl Iterator for ColorIntoIterator {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        let result = match self.index {
            0 => self.color.r,
            1 => self.color.g,
            2 => self.color.b,
            _ => return None,
        };
        self.index += 1;
        Some(result)
    }
}
