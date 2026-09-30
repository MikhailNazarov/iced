# Android support

This fork adds and maintains Android support on top of upstream
`iced`. The windowing layer is [`MikhailNazarov/winit`] (branch
`iced-android`, a fork of `iced-rs/winit` at `05b8ff17`) with
lifecycle fixes for OneUI/Samsung devices.

[`MikhailNazarov/winit`]: https://github.com/MikhailNazarov/winit

## Requirements

- Android NDK (r29 was tested; `ANDROID_NDK_ROOT` must point at it)
- Rust targets: `rustup target add aarch64-linux-android` (and/or
  `x86_64-linux-android` for the emulator)
- [`cargo-apk`](https://crates.io/crates/cargo-apk) for packaging:
  `cargo install cargo-apk`
- The cross-compilation environment variables for `cargo`/`cc-rs`:

  ```sh
  export ANDROID_HOME="$HOME/Android/Sdk"
  export ANDROID_NDK_ROOT="$ANDROID_HOME/ndk/29.0.14206865"
  export ANDROID_NDK_HOME="$ANDROID_NDK_ROOT"
  export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang"
  export CC_aarch64_linux_android="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang"
  export CXX_aarch64_linux_android="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang++"
  export AR_aarch64_linux_android="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
  ```

## Entry point

Applications are built as a `cdylib` and must expose the
`android_main` entry point:

```rust
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android_app: iced::platform::AndroidApp) {
    iced::platform::set_android_app(android_app);

    let _ = my_app::main();
}
```

With `cargo-apk` this goes into `src/lib.rs`, and `Cargo.toml` gets
a `[lib]` section with `crate-type = ["cdylib"]` plus the manifest
metadata:

```toml
[package.metadata.android]
package = "com.example.my_app"
apk_name = "my-app"
build_targets = ["aarch64-linux-android"]

[package.metadata.android.sdk]
min_sdk_version = 24
target_sdk_version = 34

[package.metadata.android.application]
label = "My App"
```

Then build and install:

```sh
cargo apk build --target aarch64-linux-android
adb install -r target/debug/apk/my-app.apk
```

Logging is available through any `log` implementation; initialize
[`android_logger`](https://crates.io/crates/android_logger) inside
`android_main` to see `iced` logs in `adb logcat`.

## Platform features

- **Theming**: applications start in the system theme and follow its
  changes; the status and navigation bars are painted with the
  background color of the window and their icon appearance matches
  the theme automatically.
- **Safe area insets**: `window::insets_events()` reports the areas
  obstructed by system UI (status bar, navigation bar, display
  cutout) and `window::insets(id)` queries them on demand. Applications
  should use them to pad their root widget, since the window is drawn
  edge-to-edge.
- **Clipboard**: text read/write through the system `ClipboardManager`.
- **System fonts**: the fonts of the device (`/system/fonts`) are
  loaded automatically; a default sans-serif family is picked from the
  available ones. Applications can still embed their own fonts.
- **Back button**: requests the window to close, like on desktop;
  applications can opt out with
  `window::Settings::exit_on_close_request = false` and subscribe to
  `window::close_requests()` to handle it (e.g. in-app navigation).
- **Rotation and theme changes**: the system recreates the activity;
  the application restarts within the same process, so the application
  state does not survive — persist it yourself if needed.

## Known limitations

- IME support is basic: the soft keyboard shows and delivers text,
  but cursor area and input purpose are not forwarded to the system.
- The system recreates the activity on configuration changes (theme
  changes, rotation) and the application restarts; applications can
  keep their state with
  `iced::platform::{save_persisted_state, load_persisted_state}`
  (a file in the internal storage of the application; survives
  process death) or with
  `iced::platform::{save_instance_state, load_instance_state}`
  (in-process; survives recreation only).
- On some Samsung devices (OneUI), closing the application with the
  back gesture may leave a frozen frame until the system cleans it up.
