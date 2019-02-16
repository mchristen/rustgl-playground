use gl;
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::hinting::HintingOptions;
use crate::resources::Resources;

pub struct Font {
    font: Vec<font_kit::font::Font>,
    gl: gl::Gl
}

impl Font {
    pub fn from_resource(gl: &gl::Gl, resources: &Resources, path: &str) -> Font {
        Font {
            font: resources.load_font(path).unwrap(),
            gl: gl.clone(),
        }
    }
}

struct Atlas {
    nodes: nalgebra::Vector4<i32>,
    pixel_width: usize,
    pixel_height: usize,
    pixel_depth: usize,
    data_length: usize,
    data_used: usize,
}

impl Atlas {
    fn new(width: i32, height: i32, depth: i32) {
    }
}