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

    /// Passes the [`AndroidApp`] to the event loop.
    pub(crate) fn take_android_app() -> Option<AndroidApp> {
        android_app()
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
