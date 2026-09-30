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

    /// Takes the [`AndroidApp`] of the current application, if it was stored
    /// with [`set_android_app`].
    pub(crate) fn take_android_app() -> Option<AndroidApp> {
        ANDROID_APP
            .lock()
            .expect("Lock Android app")
            .take()
    }
}
