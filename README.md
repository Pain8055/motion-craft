# Motion Craft

**A high-performance Motion Graphics & Compositing software rebuilt from scratch in pure Rust for Desktop and Android (Vulkan / wgpu).**

Inspired by Adobe After Effects and modern mobile compositors, Motion Craft runs natively without a heavy browser engine or JVM, delivering 60 FPS timeline playback and hardware-accelerated rendering on Android.

---

## Features

- **Layer-Based Compositing:** Solid colors, text, images, and adjustment layers.
- **Transform Hierarchy:** Full parenting system with position, anchor point, scale, rotation, and opacity.
- **Keyframe Engine:** Sub-frame keyframe interpolation with Cubic Bezier easing curves (Graph Editor).
- **13 Blending Modes:** Normal, Multiply, Screen, Overlay, Darken, Lighten, Add, Difference, and more.
- **Native Vulkan & wgpu:** Uses Android's native `libvulkan.so` for direct GPU acceleration.
- **Cross-Platform Input:** Built-in support for touch gestures (pinch-zoom, drag) alongside physical Bluetooth/USB mouse, keyboard, and stylus.
- **Automated GitHub Actions CI:** Builds `.apk` binaries directly in the cloud for `arm64-v8a` and `x86_64`.

---

## Workspace Architecture

```text
motion-craft/
├── Cargo.toml                    # Root workspace
├── crates/
│   ├── motion-core/              # Composition, layers, bezier curves, keyframes
│   ├── motion-render/            # wgpu render pipeline & WGSL shaders
│   ├── motion-ui/                # egui timeline, graph editor, inspector, viewport
│   └── motion-app/               # Winit window loop & Android NativeActivity entry
├── android/
│   └── AndroidManifest.xml       # Hardware-accelerated Vulkan manifest
├── packaging/
│   └── build_apk.py              # Automated APK packager (cargo-ndk + aapt2)
└── .github/workflows/
    └── android.yml               # GitHub Actions cloud APK builder
```

---

## Building Locally

### Desktop (Linux, macOS, Windows)
```bash
cargo run --bin motion-craft
```

### Android APK (via cargo-ndk)
Ensure you have `cargo-ndk` installed and `ANDROID_HOME` configured:
```bash
python3 packaging/build_apk.py --abi arm64-v8a --profile release
```

---

## License
MIT OR Apache-2.0
