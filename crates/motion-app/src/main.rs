mod app;

use app::MotionApp;
use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = MotionApp::default();
    event_loop.run_app(&mut app).expect("Application loop failed");
}
