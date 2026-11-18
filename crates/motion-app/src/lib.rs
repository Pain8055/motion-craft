pub mod app;

#[cfg(target_os = "android")]
use winit::event_loop::EventLoop;
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;
#[cfg(target_os = "android")]
use winit::platform::android::EventLoopBuilderExtAndroid;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(android_app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("MotionCraft"),
    );

    std::panic::set_hook(Box::new(|info| {
        log::error!("MotionCraft PANIC: {:?}", info);
    }));

    log::info!("Starting Motion Craft Android native activity...");

    let event_loop = EventLoop::builder()
        .with_android_app(android_app)
        .build()
        .expect("Failed to build Android event loop");

    let mut app = app::MotionApp::default();
    event_loop.run_app(&mut app).expect("Android application loop failed");
}
