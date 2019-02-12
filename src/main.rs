pub mod resources;
pub mod shaders;

use std::path::Path;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::ffi::{CStr};
use glutin::dpi::*;
use glutin::GlContext;
use slog::Drain;
use slog::info;
use slog::o;

use resources::Resources;
use shaders::Program;

struct Scene {
    program_id: gl::types::GLuint,
    vertices: Vec<f32>,
    vbo_id: gl::types::GLuint,
    vao_id: gl::types::GLuint,
    gl: gl::Gl,
}

impl Scene {
    fn with_program(gl: &gl::Gl, program: &Program) -> Box<Scene> {
        let vertices: Vec<f32> = vec![
            -0.5, -0.5, 0.0, 1.0, 0.0, 0.0,
            0.5, -0.5, 0.0, 0.0, 1.0, 0.0,
            0.0, 0.5, 0.0, 0.0, 0.0, 1.0
        ];

        let mut b_id: gl::types::GLuint = 0;
        let mut a_id: gl::types::GLuint = 0;
        unsafe {
            gl.GenBuffers(1, &mut b_id);
        }
        unsafe {
            gl.BindBuffer(gl::ARRAY_BUFFER, b_id);
            gl.BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW
            );
            gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            gl.GenVertexArrays(1, &mut a_id);
        }
        
        unsafe {
            gl.BindVertexArray(a_id);
            gl.BindBuffer(gl::ARRAY_BUFFER, b_id);
            gl.EnableVertexAttribArray(0);
            gl.VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (6 * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null()
            );
            gl.EnableVertexAttribArray(1);
            gl.VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                (6 * std::mem::size_of::<f32>()) as gl::types::GLint,
                (3 * std::mem::size_of::<f32>()) as *const gl::types::GLvoid
            );
            gl.BindVertexArray(0);
            gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        return Box::new(Scene { gl: gl.clone(), program_id: program.id(), vertices: vertices, vbo_id: b_id, vao_id: a_id });
    }

    fn draw(&self) {
        unsafe {
            self.gl.BindVertexArray(self.vao_id);
            self.gl.DrawArrays(
                gl::TRIANGLES,
                0,
                3
            );
        }
    }
}

struct Game<'a> {

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

    fn new(gl: &gl::Gl, resources: &Resources, gl_window: &'a glutin::GlWindow, event_loop: &'a mut glutin::EventsLoop, log: &'a slog::Logger) -> Box<Game<'a>>  {

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

    fn run(&'_ mut self) -> Result<(), failure::Error> {
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
                let program = self.programs.get(&scene.program_id).unwrap();
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

fn main() {
    let log_decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(log_decorator).use_original_order().use_utc_timestamp().build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!("version" => "0.1.0"));
    info!(log, "I'm alive!");
    let mut event_loop = glutin::EventsLoop::new();
    let builder = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new().with_vsync(true).with_gl(glutin::GlRequest::Latest);
    let gl_window = glutin::GlWindow::new(builder, context, &event_loop).unwrap();
    gl_window.show();
    unsafe {
        gl_window.make_current().unwrap();
    }

    let gl = gl::Gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
    let data = unsafe { CStr::from_ptr(gl.GetString(gl::VERSION) as *const _).to_bytes().to_vec() };

    let version = String::from_utf8(data).unwrap();
    info!(log, "OpenGL Version {}", version);
    unsafe {
        gl.Viewport(0,0,1024,768);
        gl.ClearColor(0.3, 0.3, 0.5, 1.0);
        // gl::Enable(gl::BLEND);
        //gl::BlendFunc(gl::SRC_ALPHA, gl::SRC_COLOR);
    }
    let resources = Resources::from_relative_exe(Path::new("assets")).unwrap();
    println!("Resource path: {:?}", resources.root());
    let mut game = Game::new(&gl, &resources, &gl_window, &mut event_loop, &log);
    if let Err(e) = game.run() {
        println!("{}", failure_to_string(e));
    }

}

pub fn failure_to_string(e: failure::Error) -> String {
    use std::fmt::Write;

    let mut result = String::new();

    for (i, cause) in e.iter_chain().collect::<Vec<_>>().into_iter().rev().enumerate() {
        if i > 0 {
            let _ = writeln!(&mut result, "   Which caused the following issue:");
        }
        let _ = write!(&mut result, "{}", cause);
        if let Some(backtrace) = cause.backtrace() {
            let backtrace_str = format!("{}", backtrace);
            if backtrace_str.len() > 0 {
                let _ = writeln!(&mut result, " This happened at {}", backtrace);
            } else {
                let _ = writeln!(&mut result);
            }
        } else {
            let _ = writeln!(&mut result);
        }
    }

    result
}
