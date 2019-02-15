extern crate rustgl_render_derive;

pub mod resources;
pub mod game;
pub mod render;

use std::path::Path;
use std::ffi::{CStr};
use glutin::GlContext;
use slog::Drain;
use slog::info;
use slog::o;

use resources::Resources;
use game::Game;

fn main() {
    let log_decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(log_decorator).use_original_order().use_utc_timestamp().build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(drain, o!("version" => "0.1.0"));
    info!(log, "I'm alive!");
    let mut event_loop = glutin::EventsLoop::new();
    let builder = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new().with_vsync(true).with_gl(glutin::GlRequest::Latest);
    let gl_window = glutin::GlWindow::new(builder, context, &event_loop).unwrap();
    gl_window.show();
    unsafe {
        gl_window.make_current().unwrap();
    }

    let gl = gl::Gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
    let data = unsafe { CStr::from_ptr(gl.GetString(gl::VERSION) as *const _).to_bytes().to_vec() };

    let version = String::from_utf8(data).unwrap();
    info!(log, "OpenGL Version {}", version);
    unsafe {
        gl.Viewport(0,0,1024,768);
        gl.ClearColor(0.3, 0.3, 0.5, 1.0);
        // gl::Enable(gl::BLEND);
        //gl::BlendFunc(gl::SRC_ALPHA, gl::SRC_COLOR);
    }
    let resources = Resources::from_relative_exe(Path::new("assets")).unwrap();
    println!("Resource path: {:?}", resources.root());
    let mut game = Game::new(&gl, &resources, &gl_window, &mut event_loop, &log);
    if let Err(e) = game.run() {
        println!("{}", failure_to_string(e));
    }

}

pub fn failure_to_string(e: failure::Error) -> String {
    use std::fmt::Write;

    let mut result = String::new();

    for (i, cause) in e.iter_chain().collect::<Vec<_>>().into_iter().rev().enumerate() {
        if i > 0 {
            let _ = writeln!(&mut result, "   Which caused the following issue:");
        }
        let _ = write!(&mut result, "{}", cause);
        if let Some(backtrace) = cause.backtrace() {
            let backtrace_str = format!("{}", backtrace);
            if backtrace_str.len() > 0 {
                let _ = writeln!(&mut result, " This happened at {}", backtrace);
            } else {
                let _ = writeln!(&mut result);
            }
        } else {
            let _ = writeln!(&mut result);
        }
    }

    result
}
