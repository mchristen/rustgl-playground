pub mod scenes;

use glutin::dpi::LogicalSize;
use crate::render::shaders::Program;
use std::collections::HashMap;
use std::collections::VecDeque;
use crate::resources::Resources;
use slog::{info, debug, trace, warn, error};
use slog::o;
use nalgebra;

use crate::render::viewport::Viewport;
use crate::render::color_buffer::ColorBuffer;
use crate::render::font::Font;
use scenes::Scene;

pub struct Game<'a> {

    key_presses: VecDeque<glutin::VirtualKeyCode>,
    running: bool,
    window: &'a glutin::GlWindow,
    event_loop: &'a mut glutin::EventsLoop,
    scenes: Vec<Box<Scene>>,
    programs: HashMap<gl::types::GLuint, Box<Program>>,
    log: slog::Logger,
    gl: gl::Gl,
    viewport: Viewport,
    color_buffer: ColorBuffer,
    font: Font,
}

impl<'a> Game<'a> {

    pub fn new(gl: &gl::Gl, resources: &Resources, gl_window: &'a glutin::GlWindow, event_loop: &'a mut glutin::EventsLoop, log: &'a slog::Logger) -> Result<Box<Game<'a>>, failure::Error>  {

        info!(log, "Creating new Game Engine");;

        let triangle = Program::from_res(gl, resources, "shaders/triangle_test")?;
        let test_scene = Scene::with_program(gl, &triangle);
        let mut programs = HashMap::new();
        programs.insert(triangle.id(), triangle);
        let dpi = gl_window.get_hidpi_factor();
        let log = log.new(o!("module" => "game"));
        let font_log = log.new(o!("sub_module" => "fonts"));
        let size = gl_window.get_inner_size().unwrap();
        let physical_size = size.to_physical(dpi);
        let mut width: u32 = 0;
        let mut height: u32 = 0;
        match physical_size.into() {
            (w, h) => {
                width = w;
                height = h;
            }
        }
        let game = Box::new(Game {
            running: true,
            key_presses: VecDeque::new(),
            window: &gl_window,
            event_loop: event_loop,
            scenes: vec!(test_scene),
            programs: programs,
            log: log,
            gl: gl.clone(),
            viewport: Viewport::from_dimensions(width as i32, height as i32, dpi),
            color_buffer: ColorBuffer::from_color(nalgebra::Vector3::new(0.3, 0.3, 0.5)),
            font: Font::from_resource(gl, resources, "fonts/DigitalDream.ttf", 32.0, &font_log),
            });

        return Ok(game);
    }

    pub fn run(&'_ mut self) -> Result<(), failure::Error> {
        self.viewport.set_used(&self.gl);
        self.color_buffer.set_used(&self.gl);
        while self.is_running() {
            let mut events = vec!();
            self.event_loop.poll_events(|e| {
                events.push(e);
            });
            for event in &events {
                self.handle_event(event);
            }
            while let Some(key_press) = self.key_presses.pop_front() {
                match key_press {
                    glutin::VirtualKeyCode::Escape => {
                        self.exit();
                    },
                    _ => {
                        debug!(self.log, "Unhandled keypress: {:?}", key_press);
                    }
                }
            }
            self.color_buffer.clear(&self.gl);
            for scene in &self.scenes {
                let program = self.programs.get(&scene.program_id()).unwrap();
                program.activate();
                scene.draw();
            }
            self.window.swap_buffers().unwrap();
        }
        Ok(())
    }

    fn handle_event(&'_ mut self, e: &glutin::Event) {
        if let glutin::Event::WindowEvent { event, .. } = e {
            self.handle_window_event(event);
        }

    }

    fn is_running(&'_ self) -> bool {
        return self.running;
    } 

    fn exit(&'_ mut self) {
        self.running = false;
    }

    fn handle_window_event(&'_ mut self, event: &glutin::WindowEvent) {

        use glutin::WindowEvent::*;

        match event{
            CloseRequested => {
                println!("The close button was pressed; stopping");
                self.exit();
            },
            Resized(LogicalSize { width, height }) => {
                self.viewport.change_size(*width as i32, *height as i32);
                self.viewport.set_used(&self.gl);
                println!("The window was resized to {}x{}", width, height);
            },
            glutin::WindowEvent::KeyboardInput {
                input: glutin::KeyboardInput {
                    state: glutin::ElementState::Released,
                    virtual_keycode: Some(key),
                    modifiers,
                    ..
                },
                ..
            } => {
                self.key_presses.push_back(*key);
            },
            _ => ()
        }
    }

}
