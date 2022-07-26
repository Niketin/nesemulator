use crate::{
    render_gl::{self, Program},
    texture::Texture,
};
use egui::{epaint::Primitive, PaintCallbackInfo, Rect, TextureId};
use itertools::{self, Itertools};
use std::{cell::RefCell, collections::HashMap, ffi::CString};

type CallbackFunctionType = Box<dyn Fn(PaintCallbackInfo, &Painter) + Sync + Send>;

pub struct CallbackFn {
    f: CallbackFunctionType,
}

impl CallbackFn {
    pub fn new<F: Fn(PaintCallbackInfo, &Painter) + Sync + Send + 'static>(callback: F) -> Self {
        let f = Box::new(callback);
        CallbackFn { f }
    }
}

pub struct Painter {
    pub textures: HashMap<TextureId, Texture>,
    shader_program: Program,
    vbo: gl::types::GLuint,
    vao: gl::types::GLuint,
    screen_rect: egui::Rect,
    canvas_size: [u32; 2],
    pixels_per_point: f32,
    next_native_tex_id: u64,
}

impl Painter {
    pub fn new(screen_width: u32, screen_height: u32, pixels_per_point: f32) -> Self {
        let vert_shader = render_gl::Shader::from_vert_source(
            &CString::new(include_str!("triangle.vert")).unwrap(),
        )
        .unwrap();
        let frag_shader = render_gl::Shader::from_frag_source(
            &CString::new(include_str!("triangle.frag")).unwrap(),
        )
        .unwrap();

        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
        }

        let mut vao: gl::types::GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        }

        let shader_program = render_gl::Program::from_shaders(&[vert_shader, frag_shader]).unwrap();
        let rect = egui::vec2(screen_width as f32, screen_height as f32) / pixels_per_point;
        let screen_rect = egui::Rect::from_min_size(egui::Pos2::new(0.0, 0.0), rect);
        Self {
            textures: Default::default(),
            shader_program,
            vbo,
            vao,
            screen_rect,
            canvas_size: [screen_width, screen_height],
            pixels_per_point,
            next_native_tex_id: 0,
        }
    }

    // pub fn texture_creator(&self) -> &TextureCreator<sdl2::video::WindowContext> {
    //     &self.texture_creator
    // }
    // pub fn texture_creator_mut(&mut self) -> &mut TextureCreator<sdl2::video::WindowContext> {
    //     &mut self.texture_creator
    // }
    pub fn paint_and_update_textures(
        &mut self,
        clipped_primitives: &[egui::ClippedPrimitive],
        textures_delta: egui::TexturesDelta,
        pixels_per_point: f32,
    ) {
        for (texture_id, image_delta) in textures_delta.set {
            self.set_texture(&texture_id, image_delta);
        }

        self.paint_primitives(clipped_primitives, pixels_per_point);

        for texture_id in &textures_delta.free {
            self.free_texture(texture_id);
        }
    }

    fn paint_primitives(
        &self,
        clipped_primitives: &[egui::ClippedPrimitive],
        pixels_per_point: f32,
    ) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        for egui::ClippedPrimitive {
            clip_rect,
            primitive,
        } in clipped_primitives
        {
            match primitive {
                Primitive::Mesh(mesh) => {
                    self.paint_mesh(pixels_per_point, clip_rect, mesh);
                }
                Primitive::Callback(callback) => {
                    let cbfn = if let Some(c) = callback.callback.downcast_ref::<CallbackFn>() {
                        c
                    } else {
                        panic!("Unsupported render callback")
                    };

                    if callback.rect.is_positive() {
                        let rect_min_x = pixels_per_point * callback.rect.min.x;
                        let rect_min_y = pixels_per_point * callback.rect.min.y;
                        let rect_max_x = pixels_per_point * callback.rect.max.x;
                        let rect_max_y = pixels_per_point * callback.rect.max.y;

                        let rect_min_x = rect_min_x.round() as i32;
                        let rect_min_y = rect_min_y.round() as i32;
                        let rect_max_x = rect_max_x.round() as i32;
                        let rect_max_y = rect_max_y.round() as i32;

                        unsafe {
                            gl::Viewport(
                                rect_min_x,
                                self.screen_rect.height() as i32 - rect_max_y,
                                rect_max_x - rect_min_x,
                                rect_max_y - rect_min_y,
                            );
                        }

                        let info = egui::PaintCallbackInfo {
                            viewport: callback.rect,
                            clip_rect: *clip_rect,
                            pixels_per_point,
                            screen_size_px: self.canvas_size,
                        };

                        (cbfn.f)(info, self);
                    }
                }
            }
        }
    }

    fn set_texture(&mut self, texture_id: &TextureId, image_delta: egui::epaint::ImageDelta) {
        if image_delta.is_whole() {
            // Create new texture
            let texture = Texture::from_image_delta(image_delta);
            self.textures.insert(*texture_id, texture);
        } else {
            // Update existing texture
            if let Some(t) = self.textures.get_mut(texture_id) {
                t.update(image_delta)
            }
        }
    }

    fn free_texture(&mut self, texture_id: &TextureId) {
        self.textures.remove(texture_id);
    }

    pub fn paint_custom(
        &self,
        _pixels_per_point: f32,
        clip_rect: &Rect,
        texture_id: &'static std::thread::LocalKey<RefCell<Option<egui::TextureId>>>,
    ) {
        let indices = vec![0, 1, 2, 1, 3, 2];
        let top_left = clip_rect.min;
        let top_right = epaint::pos2(clip_rect.min.x, clip_rect.max.y);
        let bottom_left = epaint::pos2(clip_rect.max.x, clip_rect.min.y);
        let bottom_right = clip_rect.max;
        let vertices = vec![
            epaint::Vertex {
                pos: top_left,
                uv: epaint::pos2(0.0, 0.0),
                color: epaint::Color32::WHITE,
            },
            epaint::Vertex {
                pos: top_right,
                uv: epaint::pos2(0.0, 1.0),
                color: epaint::Color32::WHITE,
            },
            epaint::Vertex {
                pos: bottom_left,
                uv: epaint::pos2(1.0, 0.0),
                color: epaint::Color32::WHITE,
            },
            epaint::Vertex {
                pos: bottom_right,
                uv: epaint::pos2(1.0, 1.0),
                color: epaint::Color32::WHITE,
            },
        ];

        texture_id.with(|x| {
            let texture_id = x.borrow().unwrap();
            let mesh = egui::Mesh {
                indices,
                vertices,
                texture_id,
            };
            self.paint_mesh(_pixels_per_point, clip_rect, &mesh);
        });
    }

    fn paint_mesh(&self, _pixels_per_point: f32, clip_rect: &Rect, mesh: &egui::Mesh) {
        debug_assert!(mesh.is_valid());
        let vertex_count = mesh.indices.len();
        let texture = self.textures.get(&mesh.texture_id).unwrap();
        let data = mesh
            .indices
            .iter()
            .flat_map(|i| {
                let v = mesh.vertices[*i as usize];
                let (c0, c1, c2, c3) = (
                    v.color.r() as f32, // / 255f32,
                    v.color.g() as f32, // / 255f32,
                    v.color.b() as f32, // / 255f32,
                    v.color.a() as f32, // / 255f32,
                );
                [v.pos.x, v.pos.y, c0, c1, c2, c3, v.uv.x, v.uv.y]
            })
            .collect_vec();
        // Copy vertex data to GPU
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                data.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW,
            );
        }

        // Bind texture
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, texture.gl_texture_id);

            //gl::TexParameteri(gl::TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        }

        unsafe {
            gl::BindVertexArray(self.vao);

            gl::EnableVertexAttribArray(0);
            gl::EnableVertexAttribArray(1);
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                (8 * std::mem::size_of::<f32>()) as i32,
                std::ptr::null::<gl::types::GLvoid>(),
            );
            gl::VertexAttribPointer(
                1,
                4,
                gl::FLOAT,
                gl::FALSE,
                (8 * std::mem::size_of::<f32>()) as i32,
                (2 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid,
            );
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                (8 * std::mem::size_of::<f32>()) as i32,
                (6 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            let clip_min_x = self.pixels_per_point * clip_rect.min.x;
            let clip_min_y = self.pixels_per_point * clip_rect.min.y;
            let clip_max_x = self.pixels_per_point * clip_rect.max.x;
            let clip_max_y = self.pixels_per_point * clip_rect.max.y;

            let clip_min_x = clip_min_x.clamp(0.0, self.screen_rect.width());
            let clip_min_y = clip_min_y.clamp(0.0, self.screen_rect.height());
            let clip_max_x = clip_max_x.clamp(clip_min_x, self.screen_rect.width());
            let clip_max_y = clip_max_y.clamp(clip_min_y, self.screen_rect.height());

            let clip_min_x = clip_min_x.round() as i32;
            let clip_min_y = clip_min_y.round() as i32;
            let clip_max_x = clip_max_x.round() as i32;
            let clip_max_y = clip_max_y.round() as i32;

            gl::Viewport(0, 0, self.canvas_size[0] as i32, self.canvas_size[1] as i32);

            gl::Enable(gl::FRAMEBUFFER_SRGB);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(
                clip_min_x,
                self.canvas_size[1] as i32 - clip_max_y,
                clip_max_x - clip_min_x,
                clip_max_y - clip_min_y,
            );

            let screen_size_uniform_location: gl::types::GLint = gl::GetUniformLocation(
                self.shader_program.id(),
                "screen_size".as_ptr() as *const gl::types::GLbyte,
            );
            self.shader_program.set_used();
            gl::Uniform2f(
                screen_size_uniform_location,
                self.screen_rect.width(),
                self.screen_rect.height(),
            );
            gl::DrawArrays(gl::TRIANGLES, 0, vertex_count as i32);

            gl::Disable(gl::SCISSOR_TEST);
            gl::Disable(gl::FRAMEBUFFER_SRGB);
            gl::Disable(gl::BLEND);
            gl::DisableVertexAttribArray(0);
            gl::DisableVertexAttribArray(1);
            gl::DisableVertexAttribArray(2);
            gl::BindVertexArray(0);
        }
    }

    pub fn register_native_texture(&mut self, native: Texture) -> egui::TextureId {
        let id = egui::TextureId::User(self.next_native_tex_id);
        self.next_native_tex_id += 1;
        self.textures.insert(id, native);
        id
    }

    pub fn replace_native_texture(&mut self, id: egui::TextureId, replacing: Texture) {
        self.textures.insert(id, replacing);
    }
}
