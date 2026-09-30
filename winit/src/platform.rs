//! Platform-specific integration.
#[cfg(target_os = "android")]
pub mod android {
    //! Integration with the Android platform.
    //!
    //! On Android, applications are built as `cdylib` libraries and started
    //! by the system through the `android_main` entry point. `winit` needs
    //! the [`AndroidApp`] provided there to create its event loop.
    //!
    //! Store it with [`set_android_app`] before running an iced program:
    //!
    //! ```ignore
    //! #[no_mangle]
    //! fn android_main(android_app: iced::platform::AndroidApp) {
    //!     iced::platform::set_android_app(android_app);
    //!
    //!     counter::main();
    //! }
    //! ```

    use crate::core::Color;
    use jni::{jni_sig, jni_str};
    use std::sync::Mutex;

    static ANDROID_APP: Mutex<Option<winit::platform::android::activity::AndroidApp>> =
        Mutex::new(None);

    /// The Android application handle provided to `android_main`.
    pub type AndroidApp = winit::platform::android::activity::AndroidApp;

    /// Stores the [`AndroidApp`] of the current application.
    ///
    /// It must be called from `android_main`, before running an iced
    /// program on Android.
    pub fn set_android_app(app: AndroidApp) {
        *ANDROID_APP.lock().expect("Lock Android app") = Some(app);
    }

    /// Returns a clone of the stored [`AndroidApp`], if any.
    pub(crate) fn android_app() -> Option<AndroidApp> {
        ANDROID_APP
            .lock()
            .expect("Lock Android app")
            .clone()
    }

    /// Returns `true` if the native window of the activity currently
    /// exists; i.e. the application is not suspended.
    pub(crate) fn native_window_exists() -> bool {
        android_app()
            .map(|app| app.native_window().is_some())
            .unwrap_or_default()
    }

    /// The latest known safe area insets of the window, in logical
    /// pixels; `[left, top, right, bottom]`.
    static WINDOW_INSETS: Mutex<[f32; 4]> = Mutex::new([0.0; 4]);

    /// Reads the safe area insets of the activity's window in logical
    /// pixels, using the given `scale_factor`.
    ///
    /// The read value is stored as the latest known one.
    #[allow(unsafe_code)] // JNI is inherently unsafe
    pub(crate) fn window_insets(scale_factor: f32) -> crate::core::window::Insets {
        let physical = read_window_insets().unwrap_or([0.0; 4]);

        let scale = if scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };

        let insets = crate::core::window::Insets {
            left: physical[0] / scale,
            top: physical[1] / scale,
            right: physical[2] / scale,
            bottom: physical[3] / scale,
        };

        *WINDOW_INSETS.lock().expect("Lock window insets") =
            [insets.left, insets.top, insets.right, insets.bottom];

        insets
    }

    /// Returns the latest known safe area insets of the window, in
    /// logical pixels.
    pub(crate) fn last_window_insets() -> crate::core::window::Insets {
        let [left, top, right, bottom] = *WINDOW_INSETS.lock().expect("Lock window insets");

        crate::core::window::Insets {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Reads the safe area insets of the activity's window, in physical
    /// pixels; `[left, top, right, bottom]`.
    #[allow(unsafe_code)] // JNI is inherently unsafe
    fn read_window_insets() -> Option<[f32; 4]> {
        let app = android_app()?;

        let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as *mut _) };

        vm.attach_current_thread(|env| -> jni::errors::Result<[f32; 4]> {
            // `android-activity` owns this reference; do not delete it
            let activity =
                unsafe { jni::objects::JObject::from_raw(env, app.activity_as_ptr() as _) };

            let window = env
                .call_method(
                    &activity,
                    jni_str!("getWindow"),
                    jni_sig!("()Landroid/view/Window;"),
                    &[],
                )?
                .l()?;

            let decor_view = env
                .call_method(
                    &window,
                    jni_str!("getDecorView"),
                    jni_sig!("()Landroid/view/View;"),
                    &[],
                )?
                .l()?;

            let window_insets = env
                .call_method(
                    &decor_view,
                    jni_str!("getRootWindowInsets"),
                    jni_sig!("()Landroid/view/WindowInsets;"),
                    &[],
                )?
                .l()?;

            let system_insets = env
                .call_method(
                    &window_insets,
                    jni_str!("getSystemWindowInsets"),
                    jni_sig!("()Landroid/graphics/Insets;"),
                    &[],
                )?
                .l()?;

            let left = env
                .get_field(&system_insets, jni_str!("left"), jni_sig!("I"))?
                .i()?;
            let top = env
                .get_field(&system_insets, jni_str!("top"), jni_sig!("I"))?
                .i()?;
            let right = env
                .get_field(&system_insets, jni_str!("right"), jni_sig!("I"))?
                .i()?;
            let bottom = env
                .get_field(&system_insets, jni_str!("bottom"), jni_sig!("I"))?
                .i()?;

            std::mem::forget(activity);

            Ok([left as f32, top as f32, right as f32, bottom as f32])
        })
        .ok()
    }

    /// Asks the system to recreate the activity.
    ///
    /// This is used when the event loop of an invocation of `android_main`
    /// is done, so that a stale activity is brought back with a fresh one.
    #[allow(unsafe_code)] // JNI is inherently unsafe
    pub(crate) fn recreate_activity() {
        let Some(app) = android_app() else {
            return;
        };

        let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as *mut _) };

        let _ = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
            // `android-activity` owns this reference; do not delete it
            let activity =
                unsafe { jni::objects::JObject::from_raw(env, app.activity_as_ptr() as _) };

            let _ = env.call_method(&activity, jni_str!("recreate"), jni_sig!("()V"), &[]);

            std::mem::forget(activity);

            Ok(())
        });
    }

    /// Passes the [`AndroidApp`] to the event loop.
    pub(crate) fn take_android_app() -> Option<AndroidApp> {
        android_app()
    }

    /// Returns the current theme of the system, if it is known.
    ///
    /// The `ui_night_mode` setting is queried directly, since neither the
    /// configuration of the activity nor the `UiModeManager` service ever
    /// reflect theme changes on some devices.
    #[allow(unsafe_code)] // JNI is inherently unsafe
    pub fn system_theme() -> Option<crate::core::theme::Mode> {
        let app = android_app()?;

        let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as *mut _) };

        let night_mode = vm
            .attach_current_thread(|env| -> jni::errors::Result<i32> {
                // `android-activity` owns this reference; do not delete it
                let activity =
                    unsafe { jni::objects::JObject::from_raw(env, app.activity_as_ptr() as _) };

                let resolver = env
                    .call_method(
                        &activity,
                        jni_str!("getContentResolver"),
                        jni_sig!("()Landroid/content/ContentResolver;"),
                        &[],
                    )?
                    .l()?;

                let secure = env.find_class(jni_str!("android/provider/Settings$Secure"))?;

                let name = env.new_string("ui_night_mode")?;

                // `Settings.Secure.getInt(resolver, "ui_night_mode", 0)`
                let night_mode = env
                    .call_static_method(
                        &secure,
                        jni_str!("getInt"),
                        jni_sig!("(Landroid/content/ContentResolver;Ljava/lang/String;I)I"),
                        &[
                            jni::JValue::Object(&resolver),
                            jni::JValue::Object(&name),
                            jni::JValue::Int(0),
                        ],
                    )?
                    .i()?;

                std::mem::forget(activity);

                Ok(night_mode)
            })
            .ok()?;

        // `UiModeManager.MODE_NIGHT_NO` and `UiModeManager.MODE_NIGHT_YES`
        Some(match night_mode {
            2 => crate::core::theme::Mode::Dark,
            1 => crate::core::theme::Mode::Light,
            _ => crate::core::theme::Mode::None,
        })
    }

    /// The `SYSTEM_UI_FLAG_LIGHT_STATUS_BAR` flag of `View`.
    const LIGHT_STATUS_BAR: i32 = 0x0000_2000;

    /// Colors the system bars with the given `background` and switches
    /// their icon appearance for the given theme.
    ///
    /// `light` must be `true` when the window uses a light theme, so that
    /// the system draws dark icons on top of it.
    #[allow(unsafe_code)] // JNI is inherently unsafe
    pub(crate) fn set_system_bars(light: bool, background: Color) {
        let Some(app) = android_app() else {
            return;
        };

        let channel = |channel: f32| (channel.clamp(0.0, 1.0) * 255.0).round() as u32;
        let color = ((channel(background.a) << 24)
            | (channel(background.r) << 16)
            | (channel(background.g) << 8)
            | channel(background.b)) as i32;

        let appearance = if light { LIGHT_STATUS_BAR } else { 0 };

        let runner = app.clone();
        runner.run_on_java_main_thread(Box::new(move || {
            let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as *mut _) };

            let result = vm.attach_current_thread(|env| -> jni::errors::Result<()> {
                // `android-activity` owns this reference; do not delete it
                let activity =
                    unsafe { jni::objects::JObject::from_raw(env, app.activity_as_ptr() as _) };

                let window = env
                    .call_method(
                        &activity,
                        jni_str!("getWindow"),
                        jni_sig!("()Landroid/view/Window;"),
                        &[],
                    )?
                    .l()?;

                let _ = env.call_method(
                    &window,
                    jni_str!("setStatusBarColor"),
                    jni_sig!("(I)V"),
                    &[jni::JValue::Int(color)],
                )?;
                let _ = env.call_method(
                    &window,
                    jni_str!("setNavigationBarColor"),
                    jni_sig!("(I)V"),
                    &[jni::JValue::Int(color)],
                )?;

                let decor = env
                    .call_method(
                        &window,
                        jni_str!("getDecorView"),
                        jni_sig!("()Landroid/view/View;"),
                        &[],
                    )?
                    .l()?;

                let _ = env.call_method(
                    &decor,
                    jni_str!("setSystemUiVisibility"),
                    jni_sig!("(I)V"),
                    &[jni::JValue::Int(appearance)],
                )?;

                std::mem::forget(activity);

                Ok(())
            });

            if let Err(error) = result {
                log::warn!("Failed to set the appearance of the system bars: {error:?}");
            }
        }));
    }
}
