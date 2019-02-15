
use gl;

pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub dpi: f64,
}
 
impl Viewport {
    pub fn from_dimensions(w: i32, h: i32, dpi: f64) -> Viewport {
        Viewport {
            x: 0,
            y: 0,
            w,
            h,
            dpi,
        }
    }

    pub fn change_size(&mut self, w: i32, h: i32) {
        self.w = w;
        self.h = h;
    }

    pub fn change_dpi(&mut self, dpi: f64) {
        self.dpi = dpi;
    }

    pub fn set_used(&self, gl: &gl::Gl) {
        unsafe {
            gl.Viewport(self.x, self.y, self.w, self.h);
        }
    }
}
