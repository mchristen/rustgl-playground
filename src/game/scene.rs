use gl;

use crate::render::shaders::Program;
use crate::render::viewport::Viewport;
use crate::game::GameTickData;
use crate::game::KeyMap;
pub trait Scene {
    fn draw(&self, shader_program: &Program, viewport: &Viewport, tick_data: &GameTickData);
    fn update(&self, key_map: &KeyMap);
    fn program_id(&self) -> gl::types::GLuint;
}