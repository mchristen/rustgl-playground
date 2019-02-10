extern crate winit;
fn main() {
    let mut event_loop = winit::EventsLoop::new();
    let builder = winit::WindowBuilder::new();
    let window = builder.build(&event_loop).unwrap();

    window.show();
    let mut exit = false;

    while !exit {
        event_loop.poll_events(|event| {
            match event{
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::CloseRequested, .. } => {
                        println!("The close button was pressed; stopping");
                        exit = true;
                },
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::Resized(winit::dpi::LogicalSize { width, height }),
                    ..
                } => {
                    println!("The window was resized to {}x{}", width, height);
                },
                _ => ()
            }
        });
    }
}
