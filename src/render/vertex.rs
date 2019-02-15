use super::types::f32_f32_f32;
use rustgl_render_derive::VertexAttribPointers;

#[derive(VertexAttribPointers)]
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Vertex {
    #[location = 0]
    pub position: f32_f32_f32,
    #[location = "1"]
    pub color: f32_f32_f32,
}
