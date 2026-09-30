use std::f32;

/// The safe area insets of a window in logical pixels.
///
/// The insets describe the areas of the window that are partially or
/// fully obstructed by system UI; like the status bar, the navigation
/// bar, or a display cutout (notch).
///
/// Applications can use them to keep their content away from those
/// areas; for instance, by using them as [`Padding`] for the root
/// widget of the window.
///
/// [`Padding`]: crate::Padding
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Insets {
    /// The inset from the left edge of the window.
    pub left: f32,
    /// The inset from the top edge of the window.
    pub top: f32,
    /// The inset from the right edge of the window.
    pub right: f32,
    /// The inset from the bottom edge of the window.
    pub bottom: f32,
}

impl Insets {
    /// Creates zero [`Insets`].
    pub const ZERO: Self = Self {
        left: 0.0,
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
    };

    /// Returns the [`Insets`] with the given top inset.
    #[must_use]
    pub fn top(mut self, top: f32) -> Self {
        self.top = top;
        self
    }

    /// Returns the [`Insets`] with the given right inset.
    #[must_use]
    pub fn right(mut self, right: f32) -> Self {
        self.right = right;
        self
    }

    /// Returns the [`Insets`] with the given bottom inset.
    #[must_use]
    pub fn bottom(mut self, bottom: f32) -> Self {
        self.bottom = bottom;
        self
    }

    /// Returns the [`Insets`] with the given left inset.
    #[must_use]
    pub fn left(mut self, left: f32) -> Self {
        self.left = left;
        self
    }
}

impl From<[f32; 4]> for Insets {
    fn from([top, right, bottom, left]: [f32; 4]) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}
