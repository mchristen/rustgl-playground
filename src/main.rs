extern crate gl;
extern crate glutin;
#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate slog_async;

use std::collections::VecDeque;
use std::collections::HashMap;
use std::ffi::{CString, CStr};
use glutin::dpi::*;
use glutin::GlContext;
use slog::Drain;

fn get_cstring_with_len(len: usize) -> CString {
    let mut buffer:Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    return unsafe { CString::from_vec_unchecked(buffer)};
}

struct Shader {
    id: gl::types::GLuint,
    gl: gl::Gl,
}

impl Drop for Shader {

    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteShader(self.id);
        }
    }
}

impl Shader {

    fn from_source(gl: &gl::Gl, source: &CStr, kind: gl::types::GLenum) -> Result<Shader, String> {
        let id = unsafe { gl.CreateShader(kind) };
        let mut result: gl::types::GLint = 1;
        unsafe {
            gl.ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
            gl.CompileShader(id);
        }
        unsafe {
            gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut result);
        }
        println!("{}", result);
        if result == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }
            let error = get_cstring_with_len(len as usize);
            unsafe {
                gl.GetShaderInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut gl::types::GLchar);
            }
            return Err(error.to_string_lossy().into_owned());
        }
        return Ok(Shader { id: id, gl: gl.clone() });
    }
    fn id(&self) -> gl::types::GLuint {
        return self.id;
    }
}

struct Program {
    id: gl::types::GLuint,
    gl: gl::Gl,
}

impl Program {
    fn from_shaders(gl: &gl::Gl, shaders: &[Shader]) -> Result<Box<Program>, String> {
        let id = unsafe { gl.CreateProgram() };
        for shader in shaders {
            println!("attaching shader id: {}", shader.id());
            unsafe {
                gl.AttachShader(id, shader.id());
            }
        }
        let mut result: gl::types::GLint = 1;
        unsafe {
            gl.LinkProgram(id);
            gl.GetProgramiv(id, gl::LINK_STATUS, &mut result);
        }

        if result == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl.GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }
            let error = get_cstring_with_len(len as usize);
            unsafe {
                gl.GetProgramInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut gl::types::GLchar);
            }
            return Err(error.to_string_lossy().into_owned());
        }


        for shader in shaders {
            unsafe {
                gl.DetachShader(id, shader.id());
            }
        }
        return Ok(Box::new(Program { id: id, gl: gl.clone()}));
    }

    fn activate(&self) {
        unsafe {
            self.gl.UseProgram(self.id);
        }
    }
}

impl Drop for Program {

    fn drop(&mut self) {
        unsafe {
            println!("Deleting program {}", self.id);
            self.gl.DeleteProgram(self.id);
        }
    }
}

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

        return Box::new(Scene { gl: gl.clone(), program_id: program.id, vertices: vertices, vbo_id: b_id, vao_id: a_id });
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

    fn new(gl: &gl::Gl, gl_window: &'a glutin::GlWindow, event_loop: &'a mut glutin::EventsLoop, log: &'a slog::Logger) -> Box<Game<'a>>  {

        info!(log, "Creating new Game Engine");;

        let v_src = include_str!("triangle_test.vert");
        let f_src = include_str!("triangle_test.frag");
        trace!(log, "Loading vertex shader source {}", v_src);
        trace!(log, "Loading fragment shader source {}", f_src);
        let v = Shader::from_source(gl, &CString::new(v_src).unwrap(), gl::VERTEX_SHADER).unwrap();
        let h = Shader::from_source(gl, &CString::new(f_src).unwrap(), gl::FRAGMENT_SHADER).unwrap();

        let triangle = Program::from_shaders(gl, &[v,h]).unwrap();
        let mut test_scene = Scene::with_program(gl, &triangle);
        let mut programs = HashMap::new();
        programs.insert(triangle.id, triangle);
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

    fn run(&'_ mut self) {
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
    let mut game = Game::new(&gl, &gl_window, &mut event_loop, &log);
    game.run();

}
