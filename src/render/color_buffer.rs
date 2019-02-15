use nalgebra;
use gl;

pub struct ColorBuffer {
    pub color: nalgebra::Vector4<f32>,
}

impl ColorBuffer {

    pub fn from_color(color: nalgebra::Vector3<f32>) -> ColorBuffer {
        ColorBuffer {
            color: color.fixed_resize::<nalgebra::U4, nalgebra::U1>(1.0),
        }
    }

    pub fn update_color(&mut self, color: nalgebra::Vector3<f32>) {
        self.color = color.fixed_resize::<nalgebra::U4, nalgebra::U1>(1.0);
    }

    pub fn set_used(&self, gl: &gl::Gl) {
        unsafe {
            gl.ClearColor(self.color.x, self.color.y, self.color.z, 1.0);
        }
    }

    pub fn clear(&self, gl: &gl::Gl) {
        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}