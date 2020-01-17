use gl;

pub struct Texture {
    gl: gl::Gl,
    texture_id: gl::types::GLuint,
}

impl Texture {
    pub fn from_data(
        gl: &gl::Gl,
        width: u32, 
        height: u32,
        data: *const std::os::raw::c_void,
    ) -> Texture {
        let mut texture_id: gl::types::GLuint = 0;
        unsafe {
            gl.GenTextures(1, &mut texture_id);
        }
        let texture = Texture {
            gl: gl.clone(),
            texture_id,
        };
        texture.activate_texture_unit(0);
        texture.bind();
        unsafe {
            gl.PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RED as gl::types::GLint,
                width as i32,
                height as i32,
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                data,
            );
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        }
        texture
    }
    pub fn bind(&self) {
        unsafe {
            self.gl.BindTexture(gl::TEXTURE_2D, self.texture_id);
        }
    }
    pub fn unbind(&self) {
        unsafe {
            self.gl.BindTexture(gl::TEXTURE_2D, 0);
        }
    }
    pub fn activate_texture_unit(&self, index: u32) {
        unsafe {
            self.gl.ActiveTexture(gl::TEXTURE0 + index);
        }
    }
}