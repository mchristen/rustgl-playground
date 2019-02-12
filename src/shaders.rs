use std::ffi::CStr;
use crate::resources::Resources;
use std::ffi::CString;
use crate::resources;
use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Failed to load resource {}", name)]
    ResourceLoad { name: String, inner: resources::Error },
    #[fail(display = "Can not determine shader type for resource {}", name)]
    CanNotDetermineShaderTypeForResource { name: String },
    #[fail(display = "Failed to compile shader {}: {}", name, message)]
    CompileError { name: String, message: String },
    #[fail(display = "Failed to link program {}: {}", name, message)]
    LinkError { name: String, message: String },
}

fn get_cstring_with_len(len: usize) -> CString {
    let mut buffer:Vec<u8> = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    return unsafe { CString::from_vec_unchecked(buffer)};
}

pub struct Shader {
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
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
    pub fn from_res(gl: &gl::Gl, resources: &Resources, name: &str) -> Result<Shader, Error> {
        const POSSIBLE_EXT: [(&str, gl::types::GLenum); 2] = [
            (".vert", gl::VERTEX_SHADER),
            (".frag", gl::FRAGMENT_SHADER),
        ];
        let shader_kind = POSSIBLE_EXT.iter()
            .find(|&&(file_extension, _)| {
                name.ends_with(file_extension)
            })
            .map(|&(_, kind)| kind)
            .ok_or_else(|| Error::CanNotDetermineShaderTypeForResource { name: name.into() })?;

        let source = resources.load_cstring(name)
            .map_err(|e| Error::ResourceLoad {
                name: name.into(),
                inner: e
            })?;

        Shader::from_source(gl, &source, shader_kind).map_err(|message| Error::CompileError {
            name: name.into(),
            message
        })
    }

    pub fn from_source(gl: &gl::Gl, source: &CStr, kind: gl::types::GLenum) -> Result<Shader, String> {
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
}

pub struct Program {
    id: gl::types::GLuint,
    gl: gl::Gl,
}

impl Program {
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
    pub fn from_res(gl: &gl::Gl, resources: &Resources, name: &str) -> Result<Box<Program>, Error> {
        const POSSIBLE_EXT: [&str; 2] = [
            ".vert",
            ".frag",
        ];
        let shaders = POSSIBLE_EXT.iter()
            .map(|file_extension| {
                Shader::from_res(gl, resources, &format!("{}{}", name, file_extension))
            }).collect::<Result<Vec<Shader>, Error>>()?;

        Program::from_shaders(gl, &shaders[..]).map_err(|message| Error::LinkError {
            name: name.into(),
            message,
        })

    }
    pub fn from_shaders(gl: &gl::Gl, shaders: &[Shader]) -> Result<Box<Program>, String> {
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

    pub fn activate(&self) {
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