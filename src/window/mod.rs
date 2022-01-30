use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};
use glium::{Display, Frame};
use glium::glutin::dpi::Size;
use glium::glutin::event::{Event, ModifiersState, MouseButton, WindowEvent};
use glium::glutin::monitor::VideoMode;
use glium::glutin::window::Fullscreen;
use crate::{Canvas, ContextBuilder, ControlFlow, ElementState, EventLoop, KeyboardInput, MouseScrollDelta, StartCause, WindowBuilder};
use crate::font::FontManager;
use crate::shaders::ShaderManager;
use crate::textures::TextureManager;

pub fn create<T, S, C, H>(title: T, inner_size: S, depth_bits: u8, mut handler: H)
    where T: Into<String>,
          S: Into<Size>,
          C: Context + 'static,
          H: Handler<C> + 'static
{
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title(title)
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .with_resizable(false)
        .with_inner_size(inner_size);

    let cb = ContextBuilder::new().with_depth_buffer(depth_bits);
    let display = Display::new(wb, cb, &event_loop).unwrap();

    let mut context = C::new(&display);
    let mut modifiers = ModifiersState::empty();

    let shaders = Rc::new(RefCell::new(ShaderManager::new(&display)));
    let fonts = Rc::new(RefCell::new(FontManager::new(&display)));
    let textures = Rc::new(RefCell::new(TextureManager::new(&display)));

    let mut last_frame = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        let fps_limit = context.get_frame_limit();
        let next_frame = Instant::now() + Duration::from_secs_f32(1.0 / fps_limit);
        *control_flow = ControlFlow::WaitUntil(next_frame);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                },
                WindowEvent::KeyboardInput { input, .. } => {
                    handler.on_keyboard_input(&mut context, input, modifiers);
                },
                WindowEvent::MouseWheel { delta, .. } => {
                    handler.on_mouse_scroll(&mut context, delta, modifiers);
                },
                WindowEvent::MouseInput { state, button, .. } => {
                    handler.on_mouse_button(&mut context, state, button, modifiers);
                },
                WindowEvent::CursorMoved { position, .. } => {
                    let dpi = display.gl_window().window().scale_factor();
                    let position = position.to_logical::<f32>(dpi);
                    handler.on_mouse_move(&mut context, position.x, position.y);
                },
                WindowEvent::ModifiersChanged(state) => {
                    modifiers = state;
                },
                WindowEvent::Resized(size) => {
                    let dpi = display.gl_window().window().scale_factor();
                    let size = size.to_logical::<f32>(dpi);
                    handler.on_resized(&mut context, size.width, size.height);
                },
                WindowEvent::HoveredFile(path) => {
                    handler.on_file_hovered(&mut context, path);
                },
                WindowEvent::DroppedFile(path) => {
                    handler.on_file_dropped(&mut context, path);
                },
                WindowEvent::HoveredFileCancelled => {
                    handler.on_file_cancelled(&mut context);
                },
                _ => {
                    *control_flow = ControlFlow::Poll;
                },
            },
            Event::NewEvents(cause) => match cause {
                StartCause::ResumeTimeReached { .. } => {},
                StartCause::Init => {},
                StartCause::WaitCancelled { .. } => {},
                _ => return,
            }
            _ => return,
        };

        let now = Instant::now();
        let time_elapsed = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;

        let mut frame = display.draw();

        let mut canvas = Canvas::new(
            display.clone(), shaders.clone(), fonts.clone(), textures.clone(), frame
        );

        handler.draw_frame(&mut context, &mut canvas, time_elapsed);

        canvas.into_inner().finish().expect("Frame finishing failed");
    });
}

pub trait Context: Sized {
    fn new(display: &Display) -> Self;

    fn get_frame_limit(&self) -> f32 {
        60.0
    }
}

pub trait Handler<C: Context>: Sized {
    fn draw_frame(&mut self, context: &mut C, canvas: &mut Canvas<Frame>, time_elapsed: f32);

    fn on_keyboard_input(&mut self, context: &mut C, input: KeyboardInput, modifiers: ModifiersState) {}

    fn on_mouse_scroll(&mut self, context: &mut C, delta: MouseScrollDelta, modifiers: ModifiersState) {}

    fn on_mouse_button(&mut self, context: &mut C, state: ElementState, button: MouseButton, modifiers: ModifiersState) {}

    fn on_mouse_move(&mut self, context: &mut C, x: f32, y: f32) {}

    fn on_resized(&mut self, context: &mut C, width: f32, height: f32) {}

    fn on_file_hovered(&mut self, context: &mut C, path: PathBuf) {}

    fn on_file_dropped(&mut self, context: &mut C, path: PathBuf) {}

    fn on_file_cancelled(&mut self, context: &mut C) {}
}