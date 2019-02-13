
use crate::render::shaders::Program;

pub struct Scene {
    program_id: gl::types::GLuint,
    vertices: Vec<f32>,
    vbo_id: gl::types::GLuint,
    vao_id: gl::types::GLuint,
    gl: gl::Gl,
}

impl Scene {
    pub fn program_id(&self) -> &gl::types::GLuint {
        return &self.program_id;
    }

    pub fn with_program(gl: &gl::Gl, program: &Program) -> Box<Scene> {
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

    pub fn draw(&self) {
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
