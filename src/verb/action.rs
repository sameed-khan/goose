use crate::errors::{OutOfBoundsError, UIActionTimeOutError};
use autopilot::bitmap::capture_screen;
use autopilot::bitmap::Bitmap;
use autopilot::geometry::Rect;
use opencv::{core, core::Mat, imgproc, prelude::*};
use std::error::Error;
use std::{thread, time};

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
        roi_mat: Option<Rect>,
    ) -> Result<bool, Box<dyn Error>> {
        let (compare_before, compare_after) = if let Some(roi_mat) = roi_mat {
            // Validate ROI dimensions
            if !roi_mat.is_rect_visible(before.bounds()) {
                return Err(Box::new(OutOfBoundsError {
                    message: format!(
                        "ROI dimensions are larger than the screen: {:?}",
                        roi_mat.size
                    ),
                }));
            }

            // Extract ROI from before and after
            (
                before.clone().cropped(roi_mat)?, // TODO: reconsider for efficiency
                after.clone().cropped(roi_mat)?,
            )
        } else {
            (before.clone(), after.clone())
        };

        Ok(!compare_before.bitmap_eq(&compare_after, Some(0.0)))
    }
}

pub trait GuiVerb: GuiAction + CheckUIState {
    /// Fires the GUI verb, executing the action and waiting for the UI state to change.
    /// The thread will continue to test whether the UI state has changed every `wait_duration` milliseconds.
    /// After `timeout` milliseconds, the function will return `UIActionTimeOutError` if the UI state has not changed.
    fn fire(&self, timeout: Option<u64>, wait_duration: Option<u64>) -> Result<(), Box<dyn Error>> {
        let mut timeout = timeout.unwrap_or(1000);
        let wait_duration = wait_duration.unwrap_or(100);

        let before = self.execute()?;
        let mut after;

        // Wait for the UI state to change, with a timeout
        while timeout > 0 {
            thread::sleep(time::Duration::from_millis(wait_duration));
            after = self.get_screenshot()?;
            if self.changed_ui_state(&before, &mut after, None)? {
                return Ok(());
            }
            timeout -= 100;
        }

        return Err(Box::new(UIActionTimeOutError {
            message: "UI action timed out".to_string(),
        }));
    }
}
