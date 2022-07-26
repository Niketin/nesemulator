use egui::{epaint::ImageDelta, ColorImage, ImageData};
use itertools::Itertools;

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub image_data: Vec<u8>,
    pub bytes_per_pixel: usize,
    pub gl_texture_id: gl::types::GLuint,
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, self.gl_texture_id as *const gl::types::GLuint);
        }
    }
}

impl Texture {
    pub fn new(image_data: Vec<u8>, width: usize, height: usize, bytes_per_pixel: usize) -> Self {
        let mut gl_texture_id: gl::types::GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut gl_texture_id);
        }

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, gl_texture_id);

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::REPEAT as gl::types::GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::REPEAT as gl::types::GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as gl::types::GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR as gl::types::GLint,
            );

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as gl::types::GLint,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                image_data.as_ptr() as *const gl::types::GLvoid,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
            gl::BindTexture(gl::TEXTURE_2D, 0)
        }

        Self {
            width,
            height,
            bytes_per_pixel,
            image_data,
            gl_texture_id,
        }
    }

    pub fn from_image_delta(image_delta: ImageDelta) -> Self {
        let width = image_delta.image.width();
        let height = image_delta.image.height();
        let bytes_per_pixel = image_delta.image.bytes_per_pixel();

        let image_data: Vec<u8> = match image_delta.image {
            egui::ImageData::Color(c) => c.pixels.into_iter().flat_map(|x| x.to_array()).collect(),
            egui::ImageData::Font(f) => f
                .srgba_pixels(1.0f32)
                .into_iter()
                .flat_map(|x| x.to_array())
                .collect(),
        };

        Self::new(image_data, width, height, bytes_per_pixel)
    }

    pub fn new_empty(width: usize, height: usize) -> Self {
        let image_data: Vec<u8> = vec![[0u8, 0u8, 0u8, 1u8]; width * height]
            .into_iter()
            .flatten()
            .collect_vec();
        Self::new(image_data, width, height, 4)
    }
    pub fn update(&mut self, image_delta: ImageDelta) {
        self.bytes_per_pixel = 4;

        if image_delta.is_whole() {
            self.width = image_delta.image.width();
            self.height = image_delta.image.height();

            let pixels: Vec<u8> = match &image_delta.image {
                egui::ImageData::Color(c) => c.pixels.iter().flat_map(|x| x.to_array()).collect(),
                egui::ImageData::Font(f) => f
                    .srgba_pixels(1.0f32)
                    .into_iter()
                    .flat_map(|x| x.to_array())
                    .collect(),
            };

            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, self.gl_texture_id);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as gl::types::GLint,
                    self.width as i32,
                    self.height as i32,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    pixels.as_ptr() as *const gl::types::GLvoid,
                );
                gl::BindTexture(gl::TEXTURE_2D, 0)
            }
        } else if let Some(pos) = image_delta.pos {
            let start_x = pos[0];
            let start_y = pos[1];

            let im_width = image_delta.image.width();
            let im_height = image_delta.image.height();

            let pixels = match image_delta.image {
                egui::ImageData::Color(c) => c.pixels,
                egui::ImageData::Font(_) => todo!(),
            };

            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, self.gl_texture_id);
                gl::TexSubImage2D(
                    gl::TEXTURE_2D,
                    0,
                    start_x as i32,
                    start_y as i32,
                    im_width as i32,
                    im_height as i32,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    pixels.as_ptr() as *const gl::types::GLvoid,
                );
            }
        }
    }

    pub fn update_from_rgb_pixel_data(&mut self, pixel_data: &[u8]) {
        let pixels = pixel_data
            .chunks_exact(3)
            .map(|x| epaint::Color32::from_rgb(x[0], x[1], x[2]))
            .collect_vec();
        let color_image = ColorImage {
            size: [self.width, self.height],
            pixels,
        };
        let image = ImageData::Color(color_image);
        let image_delta = ImageDelta {
            image,
            filter: egui::TextureFilter::Nearest,
            pos: None,
        };
        self.update(image_delta);
    }
}
