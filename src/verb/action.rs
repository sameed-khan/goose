use crate::errors::OutOfBoundsError;
use autopilot::bitmap::capture_screen;
use autopilot::bitmap::Bitmap;
use autopilot::geometry::Rect;
use std::error::Error;

/// Defines the behavior of a GUI verb.
pub trait GuiAction {
    /// The action or series of actions to be executed.
    /// This set of actions will change the UI state.
    /// Returns a screenshot of the UI BEFORE the action is executed to be used for comparison with
    /// the screenshot AFTER the action is executed.
    fn execute(&self) -> Result<Bitmap, Box<dyn Error>>;
}

/// Checks whether implementing type changed the UI state.
/// All GUI verbs are composed of a series of actions that change the UI state.
/// The next verb executed will await a UI state change before proceeding.
/// All GUI verbs must implement this trait, which compares a screenshot before and after verb execution.
pub trait CheckUIState {
    fn get_screenshot(&self) -> Result<Bitmap, Box<dyn Error>> {
        let screen = capture_screen()?;
        Ok(screen)
    }
    fn changed_ui_state(
        &self,
        before: &Bitmap,
        after: &Bitmap,
        roi: Option<Rect>,
    ) -> Result<bool, Box<dyn Error>> {
        println!("Checking UI state change inside function...");
        let (compare_before, compare_after) = if let Some(roi) = roi {
            // Validate ROI dimensions
            if !before.bounds().is_rect_visible(roi) {
                return Err(Box::new(OutOfBoundsError {
                    message: format!("ROI dimensions are larger than the screen: {:?}", roi.size),
                }));
            }

            // Extract ROI from before and after
            (
                before.clone().cropped(roi)?, // TODO: reconsider for efficiency
                after.clone().cropped(roi)?,
            )
        } else {
            (before.clone(), after.clone())
        };
        Ok(!compare_before.bitmap_eq(&compare_after, Some(0.1)))
    }
}

pub trait GuiVerb: GuiAction + CheckUIState {
    /// Fires the GUI verb, executing the action and waiting for the UI state to change.
    /// The thread will continue to test whether the UI state has changed every `wait_duration` milliseconds.
    /// After `timeout` milliseconds, the function will return `UIActionTimeOutError` if the UI state has not changed.
    /// ## Parameters
    /// * `timeout`: Optional. The maximum time in ms to wait for the UI state to change after the action. Default is 1000ms.
    /// * `wait_duration`: Optional. The time in ms to wait between checking the UI state. Default is 100ms.
    /// * `check_zone`: Optional. The region of interest to check for UI state change. Default is the entire screen.
    /// Passing a `check_zone` is highly recommended since it is likely something unrelated to the action is happening elsewhere on the screen.
    fn fire(&self, timeout: Option<u64>, wait_duration: Option<u64>) -> Result<(), Box<dyn Error>>;
    fn get_check_zone(&self) -> Rect;
}
