use std::detect::__is_feature_detected::sha;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use cgmath::{Deg, Matrix4, SquareMatrix, vec3};
use glium::{Blend, Depth, DepthTest, Display, DrawParameters, Frame, IndexBuffer, Program, Surface, Texture2d, uniform, VertexBuffer};
use glium::glutin::ContextBuilder;
use glium::glutin::dpi::LogicalSize;
use glium::glutin::event::{ElementState, KeyboardInput, ModifiersState, MouseButton, MouseScrollDelta, StartCause, VirtualKeyCode};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::index::PrimitiveType;
use glium::texture::SrgbTexture2d;
use glium::uniforms::MagnifySamplerFilter;
use rfd::FileDialog;
use crate::font::{FontParameters, TextAlignHorizontal, TextAlignVertical};
use crate::render::Canvas;

use crate::window::{Context, Handler};

#[macro_use]
extern crate glium;

mod window;
mod shaders;
mod textures;
mod render;
mod font;

struct WindowContext {
    start: Instant,
    display: Arc<Display>,
    width: f32,
    height: f32,
    color: [f32; 3],
    mouse: [f32; 2],
    words: Vec<String>,
    speedpsec: f32,
    wordind: f32,
    dropped: Option<PathBuf>
}

impl Context for WindowContext {
    fn new(display: &Display) -> Self {
        let dpi = display.gl_window().window().scale_factor();
        let size = display.gl_window().window().inner_size().to_logical::<f32>(dpi);


        Self {
            start: Instant::now(),
            display: Arc::new(display.clone()),
            width: size.width,
            height: size.height,
            mouse: [0.0, 0.0],
            color: [1.0, 0.0, 0.0],
            words: vec!["Ожидается файл".to_owned()],
            speedpsec: 0.0,
            wordind: 0.0,
            dropped: None
        }
    }
}

struct WindowHandler;

impl Handler<WindowContext> for WindowHandler {
    fn draw_frame(&mut self, context: &mut WindowContext, canvas: &mut Canvas<Frame>, time_elapsed: f32) {
        canvas.clear((0.0, 0.0, 0.0, 1.0), 1.0);
        // let r = time_elapsed.sin() * 0.5 + 0.5;
        // let g = (time_elapsed + 5.0).sin() * 0.5 + 0.5;
        // let b = (time_elapsed + 10.0).sin() * 0.5 + 0.5;

        let (x, y) = canvas.dimensions();

        // let shader = canvas.shaders().borrow().default();
        // let uniforms = uniform! {
        //     mat: Into::<[[f32; 4]; 4]>::into(canvas.viewport())
        // };
        // let params = DrawParameters::default();
        //
        // canvas.rect([0.0, 0.0, 400.0, 10.0], [0.0, 1.0, 0.0, 1.0], &*shader, &uniforms, &params);

        if let Some(file) = context.dropped.as_ref(){
            if let Some(ext) = file.extension(){
                if ext == "txt"{
                    canvas.text(format!("Файл корректный!"), x/2.0, y-800.0, &FontParameters {
                        color: [0.0, 1.0, 0.0, 1.0],
                        size: 144,
                        align_horizontal: TextAlignHorizontal::Center,
                        align_vertical: TextAlignVertical::Center,
                        .. Default::default()
                    });
                    return;
                }
            }

            canvas.text(format!("Поддерживается только файл в формате .txt"), x/2.0, y-800.0, &FontParameters {
                color: [1.0, 0.0, 0.0, 1.0],
                size: 144,
                align_horizontal: TextAlignHorizontal::Center,
                align_vertical: TextAlignVertical::Center,
                .. Default::default()
            });

        }
        else{
            context.wordind += context.speedpsec*time_elapsed;
            let index = (context.wordind) as usize;
            let word = &context.words[index % context.words.len()];
            let mut speedpmin = context.speedpsec*60.0;


            canvas.text(format!("{} слов в минуту", speedpmin), x-1100.0, y-800.0, &FontParameters {
                color: [1.0, 1.0, 1.0, 1.0],
                size: 144,
                align_horizontal: TextAlignHorizontal::Center,
                align_vertical: TextAlignVertical::Center,
                .. Default::default()
            });
            canvas.text(word, x / 2.0, y -350.0, &FontParameters {
                color: [1.0, 1.0, 1.0, 1.0],
                size: 200,
                align_horizontal: TextAlignHorizontal::Center,
                align_vertical: TextAlignVertical::Center,
                .. Default::default()
            });
        }


    }

    fn on_resized(&mut self, context: &mut WindowContext, width: f32, height: f32) {
        context.width = width;
        context.height = height;
    }

    // fn on_mouse_scroll(&mut self, context: &mut WindowContext, delta: MouseScrollDelta, modifiers: ModifiersState) {
    //     match delta {
    //         MouseScrollDelta::LineDelta(_, y) => {
    //
    //         }
    //         _ => {}
    //     }
    // }

    fn on_mouse_scroll(&mut self, context: &mut WindowContext, delta: MouseScrollDelta, modifiers: ModifiersState) {
        match delta {
            MouseScrollDelta::LineDelta(_, y) => {
                context.speedpsec += y/4.0;
                if context.speedpsec < 0.0{
                    context.speedpsec = 0.0
                }
                if context.speedpsec > 10.0{
                    context.speedpsec = 10.0
                }
            }
            _ => {}
        }
    }

    fn on_mouse_button(&mut self, context: &mut WindowContext, state: ElementState, button: MouseButton, modifiers: ModifiersState) {
        if button == MouseButton::Left && state == ElementState::Pressed{
            context.speedpsec = 0.0
        }
        if button == MouseButton::Right && state == ElementState::Pressed{
            context.speedpsec = 0.0;
            let file = FileDialog::new()
                .add_filter("Текстовый файл", &["txt"])
                .pick_file();
            if let Some(file) = file{
                load_file(context, file);
            }
        }

    }

    fn on_mouse_move(&mut self, context: &mut WindowContext, x: f32, y: f32) {
        context.mouse = [x, y];
    }

    fn on_keyboard_input(&mut self, context: &mut WindowContext, input: KeyboardInput, modifiers: ModifiersState) {
        if let Some(key) = input.virtual_keycode {
            if key == VirtualKeyCode::Escape && input.state == ElementState::Pressed {
                std::process::exit(0);
            }
        }

    }
    fn on_file_hovered(&mut self, context: &mut WindowContext, path: PathBuf) {
        context.dropped = Some(path);
    }
    fn on_file_cancelled(&mut self, context: &mut WindowContext) {
        context.dropped = None;
    }
    fn on_file_dropped(&mut self, context: &mut WindowContext, path: PathBuf) {
        context.dropped = None;
        load_file(context, path);
    }
}
fn load_file(context: &mut WindowContext, path: PathBuf){
    let input = File::open(path).unwrap();
    let buffered = BufReader::new(input);
    let mut words= Vec::new();
    for line in buffered.lines(){
        let line = line.unwrap();
        if !line.is_empty(){
            for word in line.split(' '){
                words.push(word.to_owned());
            }
        }
    }
    context.words = words;
}
fn main() {
    window::create("SpeedReading", LogicalSize::new(800, 600), 24, WindowHandler);
}