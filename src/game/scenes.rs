
use crate::render::shaders::Program;
use crate::render::vertex::Vertex;
use crate::render::array_buffer::ArrayBuffer;
use crate::render::array_buffer::VertexArray;

pub struct Scene {
    program_id: gl::types::GLuint,
    vbo: ArrayBuffer,
    vao: VertexArray,
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

        let vao = VertexArray::new(gl);
        vao.bind();
        let vbo = ArrayBuffer::new(gl);

        vbo.bind();
        vbo.upload_data(&vertices);
        Vertex::vertex_attrib_pointers(gl);
        vbo.unbind();
        vao.unbind();

        return Box::new(Scene { gl: gl.clone(), program_id: program.id(), vbo: vbo, vao: vao });
    }

    pub fn draw(&self) {
        unsafe {
            self.vao.bind();
            self.gl.DrawArrays(
                gl::TRIANGLES,
                0,
                3
            );
            self.vao.unbind();
        }
    }
}
