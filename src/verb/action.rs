use crate::errors::{OutOfBoundsError, UIActionTimeOutError};
use crate::nav::coordinate::ScreenRect;
use autopilot::bitmap;
use autopilot::bitmap::Bitmap;
use autopilot::geometry::{Point, Rect};
use std::error::Error;
use std::time::{Duration, Instant};

/// Defines the behavior of a GUI verb.
pub trait GuiAction {
    /// The action or series of actions to be executed.
    /// This set of actions will change the UI state.
    /// Returns a screenshot of the UI BEFORE the action is executed to be used for comparison with
    /// the screenshot AFTER the action is executed.
    fn execute(&self) -> Result<Bitmap, Box<dyn Error>>;
}

/// Inspects / polls against current UI state
/// Checks whether UI state has achieved a 'desired state'.
/// This could be either a lack of change in UI state (i.e: check all is stable before proceeding with GuiVerb action)
/// or a change in UI state (i.e: check that the UI state has changed as expected after GuiVerb action).
/// Parameters:
/// * `timeout`: The maximum time ms to wait for the UI state to achieve desired state
/// * `is_same`: Boolean representing whether the UI state should be the same or different from the `before` screenshot.
/// * `before`: Optional. A screenshot to compare current UI state against. If not provided, a screenshot will be taken.
/// * `roi`: Optional. Region of interest to check for UI state change. Default is the entire screen.
/// Returns:
/// * `Ok(())` if the UI state has achieved the desired state. Errors on timeout.
pub trait CheckUIState {
    fn check_ui_state(
        &self,
        timeout: u64,
        is_same: bool,
        before: Option<Bitmap>,
        roi: Option<ScreenRect>,
    ) -> Result<(), Box<dyn Error>> {
        let mut timeout_duration = Duration::from_millis(timeout);
        let before = before.unwrap_or(bitmap::capture_screen()?);
        let roi = roi.unwrap_or(ScreenRect::default());

        // Validate ROI dimensions
        if !before.bounds().is_rect_visible(roi.rect) {
            return Err(Box::new(OutOfBoundsError {
                message: format!(
                    "ROI dimensions: {:?} are larger than the screenshot input: {:?}",
                    roi.rect.size,
                    before.bounds()
                ),
            }));
        }

        while timeout_duration > Duration::from_millis(0) {
            let start = Instant::now();

            let mut after = bitmap::capture_screen()?;

            let (before_roi, after_roi) = (
                before.clone().cropped(roi.rect)?, // TODO: reconsider for efficiency
                after.cropped(roi.rect)?,
            );

            if before_roi.bitmap_eq(&after_roi, Some(0.1)) == is_same {
                return Ok(());
            }

            let elapsed = start.elapsed();
            timeout_duration = timeout_duration
                .checked_sub(elapsed)
                .unwrap_or_else(|| Duration::from_millis(0));
        }
        return Err(Box::new(UIActionTimeOutError {
            message: format!(
                "UI action timed out after {}ms; is_same: {}",
                timeout, is_same
            ),
        }));
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
    fn fire(&self, timeout: Option<u64>) -> Result<(), Box<dyn Error>>;
}
