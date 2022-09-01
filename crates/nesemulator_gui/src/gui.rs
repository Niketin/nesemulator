extern crate sdl2;

use egui::{Pos2, RawInput, Rect, Vec2};
use emulator::ppu::display::Display;
use emulator::{Button, Emulator};
use log::{debug, info};
use sdl2::controller::GameController;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use sdl2::video::{GLContext, Window};
use sdl2::{Sdl, VideoSubsystem};
use sdl2_egui::{CallbackFn, Painter, Texture};
use std::sync::Arc;
use std::{cell::RefCell, collections::HashMap, env, time::Duration};

type TextureIdContainer = std::thread::LocalKey<RefCell<Option<egui::TextureId>>>;

const DEFAULT_SCREEN_WIDTH: u32 = 1200;
const DEFAULT_SCREEN_HEIGHT: u32 = 800;

trait CustomTexture {
    fn init(&'static self, painter: &mut Painter, width: usize, height: usize);
    fn update(&'static self, painter: &mut Painter, pixel_data: &[u8]);
    fn dimensions(&'static self, painter: &Painter) -> [f32; 2];
}

impl CustomTexture for TextureIdContainer {
    fn init(&'static self, painter: &mut Painter, width: usize, height: usize) {
        let texture = Texture::new_empty(width, height, egui::TextureFilter::Nearest);
        self.with(|t| {
            t.borrow_mut()
                .get_or_insert_with(|| painter.register_native_texture(texture));
        });
    }

    fn update(&'static self, painter: &mut Painter, pixel_data: &[u8]) {
        self.with(|t| {
            let texture_id = t.borrow_mut().unwrap();
            painter
                .textures
                .get_mut(&texture_id)
                .unwrap()
                .update_from_rgb_pixel_data(pixel_data);
        });
    }

    fn dimensions(&'static self, painter: &Painter) -> [f32; 2] {
        self.with(|t| {
            let texture_id = t.borrow_mut().unwrap();
            let texture = painter.textures.get(&texture_id).unwrap();
            [texture.width as f32, texture.height as f32]
        })
    }
}

pub struct Gui {
    emulator: Emulator,
    sdl_context: Sdl,
    window: Window,
    _video_subsystem: VideoSubsystem,
    gamepads: HashMap<u32, GameController>,
    painter: Painter,
    _gl_context: GLContext,
}

impl Gui {
    pub fn new() -> Self {
        let args: Vec<String> = env::args().collect();
        let emulator = Emulator::new(&args[1]);
        let sdl_context = sdl2::init().unwrap();
        let gamepads = HashMap::new();

        let video_subsystem = sdl_context.video().unwrap();

        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);
        gl_attr.set_framebuffer_srgb_compatible(true);

        let window = video_subsystem
            .window("nesemulator", DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT) //1792 + 512, 960)
            .opengl()
            .position_centered()
            .build()
            .unwrap();

        let _gl_context = window.gl_create_context().unwrap();
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

        let painter = Painter::new(DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT, 1.0);
        
        Self {
            emulator,
            sdl_context,
            window,
            _video_subsystem: video_subsystem,
            gamepads,
            painter,
            _gl_context,
        }
    }

    pub fn run(&mut self) {
        let controller_subsystem = self.sdl_context.game_controller().unwrap();

        let egui_context = egui::Context::default();
        let time_equi_start = std::time::Instant::now();

        thread_local! {
            pub static TEXTURE_GAME: RefCell<Option<egui::TextureId>>  = RefCell::new(None);
            pub static TEXTURE_PATTERN_TABLE_0: RefCell<Option<egui::TextureId>>  = RefCell::new(None);
            pub static TEXTURE_PATTERN_TABLE_1: RefCell<Option<egui::TextureId>>  = RefCell::new(None);
            pub static TEXTURE_PALETTES: RefCell<Option<egui::TextureId>>  = RefCell::new(None);
            pub static TEXTURE_NAMETABLES: RefCell<Option<egui::TextureId>>  = RefCell::new(None);
        }
        TEXTURE_GAME.init(&mut self.painter, 256, 240);
        TEXTURE_PATTERN_TABLE_0.init(&mut self.painter, 128, 128);
        TEXTURE_PATTERN_TABLE_1.init(&mut self.painter, 128, 128);
        TEXTURE_PALETTES.init(&mut self.painter, 4, 8);
        TEXTURE_NAMETABLES.init(&mut self.painter, 512, 480);

        let mut pixels_pattern_table_0 = Display::new(128, 128);
        let mut pixels_pattern_table_1 = Display::new(128, 128);
        let mut pixels_nametables = Display::new(512, 480);

        let mut event_pump = self.sdl_context.event_pump().unwrap();

        let mut show_pattern_table_0: bool = true;
        let mut show_pattern_table_1: bool = true;
        let mut show_palettes: bool = true;
        let mut show_nametables: bool = true;

        'running: loop {
            let time_frame_start = std::time::Instant::now();

            // Process input in an event loop.
            let mut events_to_egui: Vec<egui::Event> = vec![];

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::ControllerDeviceAdded { which, .. } => {
                        info!("Gamepad added index={}", which);
                        let gamepad = controller_subsystem.open(which).unwrap();
                        self.gamepads.insert(which, gamepad);
                    }
                    Event::ControllerDeviceRemoved { which, .. } => {
                        info!("Gamepad removed index={}", which);
                        self.gamepads.remove(&(which as u32));
                    }
                    Event::ControllerButtonDown { which, button, .. } => {
                        debug!("Controller button down index={} button={:?}", which, button);
                        handle_emulator_input(event, &mut self.emulator);
                    }
                    Event::ControllerButtonUp { which, button, .. } => {
                        debug!("Controller button up index={} button={:?}", which, button);
                        handle_emulator_input(event, &mut self.emulator);
                    }
                    Event::KeyDown { .. } | Event::KeyUp { .. } => {
                        handle_emulator_input(event, &mut self.emulator)
                    }
                    Event::MouseMotion { x, y, .. } => {
                        events_to_egui
                            .push(egui::Event::PointerMoved(Pos2::new(x as f32, y as f32)));
                    }
                    Event::MouseButtonDown {
                        x, y, mouse_btn, ..
                    } => {
                        let button = match mouse_btn {
                            sdl2::mouse::MouseButton::Unknown => egui::PointerButton::Primary,
                            sdl2::mouse::MouseButton::Left => egui::PointerButton::Primary,
                            sdl2::mouse::MouseButton::Middle => egui::PointerButton::Middle,
                            sdl2::mouse::MouseButton::Right => egui::PointerButton::Secondary,
                            sdl2::mouse::MouseButton::X1 => egui::PointerButton::Primary,
                            sdl2::mouse::MouseButton::X2 => egui::PointerButton::Primary,
                        };
                        events_to_egui.push(egui::Event::PointerButton {
                            pos: Pos2 {
                                x: x as f32,
                                y: y as f32,
                            },
                            button,
                            pressed: true,
                            modifiers: egui::Modifiers::default(),
                        });
                    }
                    Event::MouseButtonUp {
                        x, y, mouse_btn, ..
                    } => {
                        let button = match mouse_btn {
                            sdl2::mouse::MouseButton::Unknown => egui::PointerButton::Primary,
                            sdl2::mouse::MouseButton::Left => egui::PointerButton::Primary,
                            sdl2::mouse::MouseButton::Middle => egui::PointerButton::Middle,
                            sdl2::mouse::MouseButton::Right => egui::PointerButton::Secondary,
                            sdl2::mouse::MouseButton::X1 => egui::PointerButton::Primary,
                            sdl2::mouse::MouseButton::X2 => egui::PointerButton::Primary,
                        };
                        events_to_egui.push(egui::Event::PointerButton {
                            pos: Pos2 {
                                x: x as f32,
                                y: y as f32,
                            },
                            button,
                            pressed: false,
                            modifiers: egui::Modifiers::default(),
                        });
                    }
                    Event::MouseWheel { x, y, .. } => {
                        events_to_egui.push(egui::Event::Scroll(Vec2 {
                            x: x as f32,
                            y: y as f32,
                        }));
                    }
                    _ => {}
                }
            }

            let inputs_to_egui: egui::RawInput = RawInput {
                screen_rect: Some(Rect::from_two_pos(
                    Default::default(),
                    Pos2::new(DEFAULT_SCREEN_WIDTH as f32, DEFAULT_SCREEN_HEIGHT as f32),
                )),
                pixels_per_point: Some(1.0),
                max_texture_side: None,
                time: Some(time_equi_start.elapsed().as_secs_f64()),
                predicted_dt: 1.0 / 60.0,
                modifiers: egui::Modifiers::default(), //TODO
                events: events_to_egui,
                hovered_files: vec![],
                dropped_files: vec![],
                has_focus: true, //TODO: add real focus from events
            };

            // Run emulator until a frame is ready.
            self.emulator.step_frame();

            // Update game screen
            let ppu = self.emulator.cpu.bus.ppu.as_mut().unwrap();
            let display = &ppu.display;
            TEXTURE_GAME.update(&mut self.painter, display.get_pixels());
            // Update game screen texture
            ppu.load_pattern_table_tiles_to_display(0x0000, &mut pixels_pattern_table_0);
            ppu.load_pattern_table_tiles_to_display(0x1000, &mut pixels_pattern_table_1);
            ppu.load_nametable_tiles_to_display(&mut pixels_nametables);
            TEXTURE_PATTERN_TABLE_0.update(&mut self.painter, pixels_pattern_table_0.get_pixels());
            TEXTURE_PATTERN_TABLE_1.update(&mut self.painter, pixels_pattern_table_1.get_pixels());
            TEXTURE_PALETTES.update(&mut self.painter, &ppu.get_current_palettes_raw());
            TEXTURE_NAMETABLES.update(&mut self.painter, pixels_nametables.get_pixels());

            // Render
            let egui::FullOutput {
                platform_output: _,
                repaint_after: _,
                textures_delta,
                shapes,
            } = egui_context.run(inputs_to_egui, |ctx| {
                egui::SidePanel::left("Settings").show(ctx, |ui| {
                    ui.checkbox(&mut show_pattern_table_0, "Show pattern table 0");
                    ui.checkbox(&mut show_pattern_table_1, "Show pattern table 1");
                    ui.checkbox(&mut show_palettes, "Show palettes");
                    ui.checkbox(&mut show_nametables, "Show nametables");
                });
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        self.ui_custom_texture_panel(ui, 2.0, &TEXTURE_GAME);
                        if show_nametables {
                            self.ui_custom_texture_panel(ui, 1.0, &TEXTURE_NAMETABLES);
                        }
                    });
                    ui.horizontal(|ui| {
                        if show_pattern_table_0 {
                            self.ui_custom_texture_panel(ui, 2.0, &TEXTURE_PATTERN_TABLE_0);
                        }
                        if show_pattern_table_1 {
                            self.ui_custom_texture_panel(ui, 2.0, &TEXTURE_PATTERN_TABLE_1);
                        }
                        if show_palettes {
                            self.ui_custom_texture_panel(ui, 24.0, &TEXTURE_PALETTES);
                        }
                    });
                });
            });

            //TODO:handle platform output
            //handle_platform_output(full_output.platform_output);

            // Create triangles to paint
            let clipped_primitives = egui_context.tessellate(shapes);

            // Paint
            self.painter
                .paint_and_update_textures(&clipped_primitives, textures_delta, 1.0f32);

            self.window.gl_swap_window();

            // Sleep 1/60th of a second minus the time it takes to render a frame.
            let time_frame_end = std::time::Instant::now();
            let time_per_frame = Duration::new(0, 1_000_000_000u32 / 60);
            let time_elapsed = time_frame_end - time_frame_start;
            let time_sleep = time_per_frame.saturating_sub(time_elapsed);
            std::thread::sleep(time_sleep);
        }
    }

    fn ui_custom_texture_panel(
        &mut self,
        ui: &mut egui::Ui,
        scale: f32,
        texture: &'static TextureIdContainer,
    ) {
        let [width, height] = texture.dimensions(&self.painter);

        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let (rect, _response) = ui.allocate_exact_size(
                Vec2::new(width * scale, height * scale),
                egui::Sense::focusable_noninteractive(),
            );
            let cb = CallbackFn::new(move |_info, painter| {
                painter.paint_custom(1.0, &rect, texture);
            });
            let callback = egui::PaintCallback {
                rect,
                callback: Arc::new(cb),
            };
            ui.painter().add(callback);
        });
    }
}

fn handle_emulator_input(event: Event, emulator: &mut Emulator) {
    let button_down = match event {
        Event::KeyDown { repeat: true, .. } => return,
        Event::KeyUp { repeat: true, .. } => return,
        Event::KeyDown { .. } => true,
        Event::ControllerButtonDown { .. } => true,
        _ => false,
    };

    let button = match event {
        Event::KeyDown {
            keycode: Some(Keycode::Z),
            ..
        } => Button::A,
        Event::KeyDown {
            keycode: Some(Keycode::X),
            ..
        } => Button::B,
        Event::KeyDown {
            keycode: Some(Keycode::N),
            ..
        } => Button::Start,
        Event::KeyDown {
            keycode: Some(Keycode::M),
            ..
        } => Button::Select,
        Event::KeyDown {
            keycode: Some(Keycode::Up),
            ..
        } => Button::Up,
        Event::KeyDown {
            keycode: Some(Keycode::Down),
            ..
        } => Button::Down,
        Event::KeyDown {
            keycode: Some(Keycode::Left),
            ..
        } => Button::Left,
        Event::KeyDown {
            keycode: Some(Keycode::Right),
            ..
        } => Button::Right,

        Event::KeyUp {
            keycode: Some(Keycode::Z),
            ..
        } => Button::A,
        Event::KeyUp {
            keycode: Some(Keycode::X),
            ..
        } => Button::B,
        Event::KeyUp {
            keycode: Some(Keycode::N),
            ..
        } => Button::Start,
        Event::KeyUp {
            keycode: Some(Keycode::M),
            ..
        } => Button::Select,
        Event::KeyUp {
            keycode: Some(Keycode::Up),
            ..
        } => Button::Up,
        Event::KeyUp {
            keycode: Some(Keycode::Down),
            ..
        } => Button::Down,
        Event::KeyUp {
            keycode: Some(Keycode::Left),
            ..
        } => Button::Left,
        Event::KeyUp {
            keycode: Some(Keycode::Right),
            ..
        } => Button::Right,

        Event::ControllerButtonDown {
            button: sdl2::controller::Button::A,
            ..
        } => Button::A,
        Event::ControllerButtonDown {
            button: sdl2::controller::Button::B,
            ..
        } => Button::B,
        Event::ControllerButtonDown {
            button: sdl2::controller::Button::Start,
            ..
        } => Button::Start,
        Event::ControllerButtonDown {
            button: sdl2::controller::Button::Back,
            ..
        } => Button::Select,
        Event::ControllerButtonDown {
            button: sdl2::controller::Button::DPadUp,
            ..
        } => Button::Up,
        Event::ControllerButtonDown {
            button: sdl2::controller::Button::DPadDown,
            ..
        } => Button::Down,
        Event::ControllerButtonDown {
            button: sdl2::controller::Button::DPadLeft,
            ..
        } => Button::Left,
        Event::ControllerButtonDown {
            button: sdl2::controller::Button::DPadRight,
            ..
        } => Button::Right,

        Event::ControllerButtonUp {
            button: sdl2::controller::Button::A,
            ..
        } => Button::A,
        Event::ControllerButtonUp {
            button: sdl2::controller::Button::B,
            ..
        } => Button::B,
        Event::ControllerButtonUp {
            button: sdl2::controller::Button::Start,
            ..
        } => Button::Start,
        Event::ControllerButtonUp {
            button: sdl2::controller::Button::Back,
            ..
        } => Button::Select,
        Event::ControllerButtonUp {
            button: sdl2::controller::Button::DPadUp,
            ..
        } => Button::Up,
        Event::ControllerButtonUp {
            button: sdl2::controller::Button::DPadDown,
            ..
        } => Button::Down,
        Event::ControllerButtonUp {
            button: sdl2::controller::Button::DPadLeft,
            ..
        } => Button::Left,
        Event::ControllerButtonUp {
            button: sdl2::controller::Button::DPadRight,
            ..
        } => Button::Right,

        _ => return,
    };

    emulator.set_controller_state(button, button_down);
}
