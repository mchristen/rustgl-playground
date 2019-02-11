extern crate gl;
extern crate glutin;

use std::collections::VecDeque;
use std::ffi::{CString, CStr};
use glutin::dpi::*;
use glutin::GlContext;

fn get_cstring_with_len(len: usize) -> CString {
    let mut buffer:Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    return unsafe { CString::from_vec_unchecked(buffer)};
}

fn load_shader(source: &CStr, shader_type: gl::types::GLuint) -> Result<gl::types::GLuint, String> {

    let id = unsafe { gl::CreateShader(shader_type) };

    let mut result: gl::types::GLint = 1;
    unsafe {
        gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(id);
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut result);
    }
    if result == 0 {
        let mut len: gl::types::GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }
        let error = get_cstring_with_len(len as usize);
        unsafe {
            gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut gl::types::GLchar);
        }
        return Err(error.to_string_lossy().into_owned());
    }
    return Ok(id);
}

struct Shader {
    id: gl::types::GLuint,
}

impl Drop for Shader {

    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

impl Shader {

    fn from_source(source: &CStr, kind: gl::types::GLenum) -> Result<Shader, String> {
        let id = load_shader(source, kind)?;
        return Ok(Shader { id });
    }
    fn id(&self) -> gl::types::GLuint {
        return self.id;
    }
}
#[derive(Clone, Debug)]
struct Program {
    id: gl::types::GLuint,
}

impl Program {
    fn from_shaders(shaders: &[Shader]) -> Result<Program, String> {
        let id = unsafe { gl::CreateProgram() };
        for shader in shaders {
            println!("attaching shader id: {}", shader.id());
            unsafe {
                gl::AttachShader(id, shader.id());
            }
        }
        let mut result: gl::types::GLint = 1;
        unsafe {
            gl::LinkProgram(id);
            gl::GetProgramiv(id, gl::LINK_STATUS, &mut result);
        }

        if result == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }
            let error = get_cstring_with_len(len as usize);
            unsafe {
                gl::GetProgramInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut gl::types::GLchar);
            }
            return Err(error.to_string_lossy().into_owned());
        }


        for shader in shaders {
            unsafe {
                gl::DetachShader(id, shader.id());
            }
        }
        return Ok(Program { id: id});
    }

    fn activate(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }
}

impl Drop for Program {

    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

struct Scene {
    program: Program,
    vertices: Vec<f32>,
    vbo_id: Option<gl::types::GLuint>,
    vao_id: Option<gl::types::GLuint>,
}

impl Scene {
    fn with_program(program: &Program) -> Scene {
        let vertices: Vec<f32> = vec![
            -0.5, -0.5, 0.0,
            0.5, -0.5, 0.0,
            0.0, 0.5, 0.0
        ];
        return Scene { program: program.clone(), vertices: vertices, vbo_id: None, vao_id: None };
    }

    fn activate(&self) {
        self.program.activate();
    }

    fn vbo_id(&self) -> gl::types::GLuint {
        return self.vbo_id.unwrap();
    }

    fn vao_id(&self) -> gl::types::GLuint {
        return self.vao_id.unwrap();
    }

    fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.vao_id());
            gl::DrawArrays(
                gl::TRIANGLES,
                0,
                3
            );
        }
    }

    fn load_data(& mut self) {
        let mut b_id: gl::types::GLuint = 0;
        let mut a_id: gl::types::GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut b_id);
        }
        self.vbo_id = Some(b_id);
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_id());
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                self.vertices.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::GenVertexArrays(1, &mut a_id);
        }
        
        self.vao_id = Some(a_id);
        unsafe {
            gl::BindVertexArray(self.vao_id());
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_id());
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (3 * std::mem::size_of::<f32>()) as gl::types::GLint,
                std::ptr::null()
            );
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}

struct Game<'a> {

    key_presses: VecDeque<glutin::VirtualKeyCode>,
    running: bool,
    window: &'a glutin::GlWindow,
    event_loop: &'a mut glutin::EventsLoop,
    scenes: Vec<Scene>,
}

impl<'a> Game<'a> {

    fn init(& mut self) {
        self.window.show();
        unsafe {
            self.window.make_current().unwrap();
            gl::load_with(|symbol| self.window.get_proc_address(symbol) as *const _);
            let data = CStr::from_ptr(gl::GetString(gl::VERSION) as *const _).to_bytes().to_vec();
            let version = String::from_utf8(data).unwrap();
            println!("OpengL version {}", version);
            gl::Viewport(0,0,1024,768);
            gl::ClearColor(0.3, 0.3, 0.5, 1.0);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::SRC_COLOR);
        }

        let v = Shader::from_source(&CString::new(include_str!("triangle_test.vert")).unwrap(), gl::VERTEX_SHADER).unwrap();
        let h = Shader::from_source(&CString::new(include_str!("triangle_test.frag")).unwrap(), gl::FRAGMENT_SHADER).unwrap();
        let triangle = Program::from_shaders(&[v,h]).unwrap();
        let mut test_scene = Scene::with_program(&triangle);
        test_scene.load_data();
        println!("{} {}", test_scene.vao_id(), test_scene.vbo_id());
        self.scenes.push(test_scene);
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
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
            for scene in &self.scenes {
                scene.activate();
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
                    gl::Viewport(0,0, *width as i32, *height as i32);
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

    let mut event_loop = glutin::EventsLoop::new();
    let builder = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new().with_vsync(true).with_gl(glutin::GlRequest::Latest);
    let gl_window = glutin::GlWindow::new(builder, context, &event_loop).unwrap();
    let mut game = Game {
        running: true,
        key_presses: VecDeque::new(),
        window: &gl_window,
        event_loop: &mut event_loop,
        scenes: vec!(),
        };
    game.init();
    game.run();

}
