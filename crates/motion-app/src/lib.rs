pub mod app;

#[cfg(target_os = "android")]
use android_activity::AndroidApp;
#[cfg(target_os = "android")]
use winit::event_loop::EventLoopBuilder;
#[cfg(target_os = "android")]
use winit::platform::android::EventLoopBuilderExtAndroid;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(android_app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("MotionCraft"),
    );

    let event_loop = EventLoopBuilder::new()
        .with_android_app(android_app)
        .build()
        .expect("Failed to build Android event loop");

    let mut app = app::MotionApp::default();
    event_loop.run_app(&mut app).expect("Android application loop failed");
}
