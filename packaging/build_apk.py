#!/usr/bin/env python3
"""Build Motion Craft Android APK using cargo-ndk and Android SDK build-tools."""

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ANDROID_DIR = ROOT / "android"
BUILD_DIR = ANDROID_DIR / "build"

def run_cmd(cmd, cwd=None, env=None):
    print(f"==> Running: {' '.join(str(c) for c in cmd)}")
    res = subprocess.run(cmd, cwd=cwd, env=env)
    if res.returncode != 0:
        print(f"Error: Command failed with exit code {res.returncode}")
        sys.exit(res.returncode)

def main():
    parser = argparse.ArgumentParser(description="Build Motion Craft Android APK")
    parser.add_argument("--abi", default="arm64-v8a", help="Android ABI (default: arm64-v8a)")
    parser.add_argument("--profile", default="release", choices=["debug", "release"], help="Cargo build profile")
    args = parser.parse_args()

    android_home = os.environ.get("ANDROID_HOME") or os.environ.get("ANDROID_SDK_ROOT")
    if not android_home:
        print("Error: ANDROID_HOME or ANDROID_SDK_ROOT is required")
        sys.exit(1)

    build_tools_dir = Path(android_home) / "build-tools"
    versions = sorted([d for d in build_tools_dir.iterdir() if d.is_dir()], reverse=True)
    if not versions:
        print("Error: No build-tools found in ANDROID_HOME")
        sys.exit(1)
    tools = versions[0]
    print(f"Using Android build-tools: {tools.name}")

    android_jar = Path(android_home) / "platforms" / "android-34" / "android.jar"
    if not android_jar.exists():
        platforms = sorted((Path(android_home) / "platforms").glob("android-*"), reverse=True)
        if platforms:
            android_jar = platforms[0] / "android.jar"
        else:
            print("Error: No android.jar platform SDK found")
            sys.exit(1)

    BUILD_DIR.mkdir(parents=True, exist_ok=True)
    libs_dir = BUILD_DIR / "libs" / args.abi
    libs_dir.mkdir(parents=True, exist_ok=True)

    # 1. Compile Rust library with cargo-ndk
    cargo_flags = ["cargo", "ndk", "-t", args.abi, "-o", str(BUILD_DIR / "libs"), "build", "-p", "motion-app", "--lib"]
    if args.profile == "release":
        cargo_flags.append("--release")
    run_cmd(cargo_flags, cwd=ROOT)

    # 2. Package APK using aapt2
    apk_unaligned = BUILD_DIR / "app-unaligned.apk"
    apk_aligned = BUILD_DIR / f"motion-craft-{args.abi}.apk"

    manifest = ANDROID_DIR / "AndroidManifest.xml"
    run_cmd([
        str(tools / "aapt2"), "link",
        "-I", str(android_jar),
        "--manifest", str(manifest),
        "--min-sdk-version", "24",
        "--target-sdk-version", "34",
        "-o", str(apk_unaligned),
    ])

    # 3. Add native libraries into APK zip
    import zipfile
    so_files = list((BUILD_DIR / "libs").rglob("*.so"))
    if not so_files:
        print(f"Error: No .so files found in {BUILD_DIR / 'libs'}")
        sys.exit(1)
    with zipfile.ZipFile(apk_unaligned, "a", compression=zipfile.ZIP_DEFLATED) as z:
        for f in so_files:
            abi = f.parent.name
            target_path = f"lib/{abi}/{f.name}"
            z.write(f, target_path)
            print(f"Added {f.name} into APK at {target_path}")

    # 4. ZipAlign
    run_cmd([
        str(tools / "zipalign"), "-f", "-p", "4",
        str(apk_unaligned), str(apk_aligned)
    ])
    if apk_unaligned.exists():
        apk_unaligned.unlink()

    # 5. Sign with debug keystore
    keystore = BUILD_DIR / "debug.keystore"
    if not keystore.exists():
        run_cmd([
            "keytool", "-genkeypair", "-keystore", str(keystore),
            "-storepass", "android", "-keypass", "android",
            "-alias", "androiddebugkey", "-dname", "CN=Android Debug,O=Android,C=US",
            "-keyalg", "RSA", "-keysize", "2048", "-validity", "10000"
        ])

    run_cmd([
        str(tools / "apksigner"), "sign",
        "--ks", str(keystore),
        "--ks-pass", "pass:android",
        "--ks-key-alias", "androiddebugkey",
        "--key-pass", "pass:android",
        str(apk_aligned)
    ])

    print(f"\n🎉 Successfully built and signed APK: {apk_aligned}")

if __name__ == "__main__":
    main()
