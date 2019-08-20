struct Display {
    width: u16,
    height: u16,
    array: Option<Vec<Pixel>>
}

impl Display {
    pub fn new() -> Display {
        Display {
            width: 256,
            height: 240,
            array: None
        }
    }

    pub fn init(&mut self) {
        self.array = Some(vec![Pixel::new(); width*height]);
    }

    pub fn get_pixel(&self, x: u16, y: u16) -> &Pixel {
        self.array[self.width * y + x]
    }

    pub fn get_pixels(&self, x: u16, y: u16) -> &Vec<Pixel> {
        self.array
    }
}

struct Pixel {
    red: u8,
    green: u8,
    blue: u8
}

impl Pixel {
    fn new() -> Pixel {
        Pixel { red: 0, green: 0, blue: 0}
    }

    fn new_rgb(red: u8, green: u8, blue: u8) -> Pixel {
        Pixel { red, green, blue }
    }


}