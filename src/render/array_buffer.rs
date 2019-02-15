use gl;


pub trait BufferType {
    const BUFFER_TYPE: gl::types::GLuint;
}

pub struct BufferTypeArray;
impl BufferType for BufferTypeArray {
    const BUFFER_TYPE: gl::types::GLuint = gl::ARRAY_BUFFER;
}

pub struct BufferTypeElementArray;
impl BufferType for BufferTypeElementArray {
    const BUFFER_TYPE: gl::types::GLuint = gl::ELEMENT_ARRAY_BUFFER;
}

pub struct Buffer<B>
where B: BufferType {
    vbo_id: gl::types::GLuint,
    gl: gl::Gl,
    _marker: ::std::marker::PhantomData<B>,
}

impl<B> Buffer<B>
where B: BufferType {
    pub fn new(gl: &gl::Gl) -> ArrayBuffer {
        let mut vbo_id: gl::types::GLuint = 0;
        unsafe {
            gl.GenBuffers(1, &mut vbo_id);
        }
        ArrayBuffer {
            vbo_id,
            gl: gl.clone(),
            _marker: ::std::marker::PhantomData,
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindBuffer(B::BUFFER_TYPE, self.vbo_id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.BindBuffer(B::BUFFER_TYPE, 0);
        }
    }

    pub fn upload_data<T>(&self, data: &[T]) {
        unsafe {
            self.gl.BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * std::mem::size_of::<T>()) as gl::types::GLsizeiptr,
                data.as_ptr() as *const gl::types::GLvoid,
                gl::STATIC_DRAW
            );
        }
    }
}

impl<B> Drop for Buffer<B>
where B: BufferType {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteBuffers(1, &mut self.vbo_id);
        }
    }
}

pub type ArrayBuffer = Buffer<BufferTypeArray>;
pub type ElementArrayBuffer = Buffer<BufferTypeElementArray>;

pub struct VertexArray {
    gl: gl::Gl,
    vao_id: gl::types::GLuint,
}

impl VertexArray {
    pub fn new(gl: &gl::Gl) -> VertexArray {
        let mut vao_id: gl::types::GLuint = 0;
        unsafe {
            gl.GenVertexArrays(1, &mut vao_id);
        }

        VertexArray {
            gl: gl.clone(),
            vao_id,
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindVertexArray(self.vao_id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.BindVertexArray(0);
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteVertexArrays(1, &mut self.vao_id);
        }
    }
}