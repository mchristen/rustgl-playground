pub mod scenes;

use glutin::dpi::LogicalSize;
use crate::render::shaders::Program;
use std::collections::HashMap;
use std::collections::VecDeque;
use crate::resources::Resources;
use slog::info;
use slog::o;

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
}

impl<'a> Game<'a> {

    pub fn new(gl: &gl::Gl, resources: &Resources, gl_window: &'a glutin::GlWindow, event_loop: &'a mut glutin::EventsLoop, log: &'a slog::Logger) -> Box<Game<'a>>  {

        info!(log, "Creating new Game Engine");;

        let triangle = Program::from_res(gl, resources, "shaders/triangle_test").unwrap();
        let test_scene = Scene::with_program(gl, &triangle);
        let mut programs = HashMap::new();
        programs.insert(triangle.id(), triangle);
        let log = log.new(o!("module" => "game"));
        let game = Box::new(Game {
            running: true,
            key_presses: VecDeque::new(),
            window: &gl_window,
            event_loop: event_loop,
            scenes: vec!(test_scene),
            programs: programs,
            log: log,
            gl: gl.clone(),
            });

        return game;
    }

    pub fn run(&'_ mut self) -> Result<(), failure::Error> {
        while self.is_running() {
            let mut events = vec!();
            self.event_loop.poll_events(|e| {
                events.push(e);
            });
            for event in &events {
                self.handle_event(event);
            }
            while let Some(key_press) = self.key_presses.pop_front() {
                println!("Key Pressed: {:?}", key_press);
            }
            unsafe {
                self.gl.Clear(gl::COLOR_BUFFER_BIT);
            }
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
                unsafe {
                    self.gl.Viewport(0,0, *width as i32, *height as i32);
                }
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
