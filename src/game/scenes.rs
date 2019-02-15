
use crate::render::shaders::Program;
use crate::render::vertex::Vertex;

pub struct Scene {
    program_id: gl::types::GLuint,
    vertices: Vec<Vertex>,
    vbo_id: gl::types::GLuint,
    vao_id: gl::types::GLuint,
    gl: gl::Gl,
}

impl Scene {
    pub fn program_id(&self) -> &gl::types::GLuint {
        return &self.program_id;
    }

    pub fn with_program(gl: &gl::Gl, program: &Program) -> Box<Scene> {
        let vertices: Vec<Vertex> = vec![
            Vertex { position: (-0.5, -0.5, 0.0).into(), color: (1.0, 0.0, 0.0).into() },
            Vertex { position: (0.5, -0.5, 0.0).into(), color: (0.0, 1.0, 0.0).into() },
            Vertex { position: (0.0, 0.5, 0.0).into(), color: (0.0, 0.0, 1.0).into() }
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
                (vertices.len() * std::mem::size_of::<Vertex>()) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW
            );
            gl.BindBuffer(gl::ARRAY_BUFFER, 0);
            gl.GenVertexArrays(1, &mut a_id);
        }
        
        unsafe {
            gl.BindVertexArray(a_id);
            gl.BindBuffer(gl::ARRAY_BUFFER, b_id);
            Vertex::vertex_attrib_pointers(gl);
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
