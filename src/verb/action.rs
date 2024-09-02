use crate::errors::{OutOfBoundsError, UIActionTimeOutError};
use autopilot::bitmap::capture_screen;
use opencv::{core, core::Mat, imgproc, prelude::*};
use std::error::Error;
use std::{thread, time};

/// Defines the behavior of a GUI verb.
pub trait GuiAction {
    /// The action or series of actions to be executed.
    /// This set of actions will change the UI state.
    /// Returns a screenshot of the UI BEFORE the action is executed to be used for comparison with
    /// the screenshot AFTER the action is executed.
    fn execute(&self) -> Result<Mat, Box<dyn Error>>;
}

/// Checks whether implementing type changed the UI state.
/// All GUI verbs are composed of a series of actions that change the UI state.
/// The next verb executed will await a UI state change before proceeding.
/// All GUI verbs must implement this trait, which compares a screenshot before and after verb execution.
pub trait CheckUIState {
    fn get_screenshot(&self) -> Result<Mat, Box<dyn Error>> {
        let screen = capture_screen()?;
        let width = screen.size.width as i32;
        let height = screen.size.height as i32;
        let raw_pixels = screen.image.raw_pixels();

        // Create a Mat from the raw pixels
        let bgr_mat = unsafe {
            Mat::new_nd_with_data_unsafe_def(
                &[height, width, 3],
                core::CV_8UC3,
                raw_pixels.as_ptr() as *mut std::ffi::c_void,
            )?
        };

        // let bgr_mat = Mat::new_rows_cols_with_data(height, width, &raw_pixels)?.try_clone()?;

        // Convert from RGB to BGR (OpenCV uses BGR by default)
        let mut opencv_mat = Mat::default();
        imgproc::cvt_color(&bgr_mat, &mut opencv_mat, imgproc::COLOR_RGB2BGR, 0)?;

        Ok(opencv_mat)
    }
    fn changed_ui_state(
        &self,
        before: &Mat,
        after: &Mat,
        roi_mat: Option<&Mat>,
    ) -> Result<bool, Box<dyn Error>> {
        let (compare_before, compare_after) = if let Some(roi_mat) = roi_mat {
            // Validate ROI dimensions
            if roi_mat.size()?.width > before.size()?.width
                || roi_mat.size()?.height > before.size()?.height
            {
                return Err(Box::new(OutOfBoundsError {
                    message: format!(
                        "ROI dimensions are larger than the before image: {:?}",
                        roi_mat.size()?
                    ),
                }));
            }

            // Find non-zero elements in ROI to get the bounding rectangle
            let non_zero_rect = imgproc::bounding_rect(&roi_mat)?;

            // Extract ROI from before and after
            (
                (Mat::roi(before, non_zero_rect)?).try_clone()?,
                (Mat::roi(after, non_zero_rect)?).try_clone()?,
            )
        } else {
            (before.try_clone()?, after.try_clone()?)
        };

        // Compare the images
        // OpenCV only compares grayscale images
        let mut diff = Mat::default();
        let mut graycompare_before = Mat::default();
        let mut graycompare_after = Mat::default();
        imgproc::cvt_color(
            &compare_before,
            &mut graycompare_before,
            imgproc::COLOR_BGR2GRAY,
            0,
        )?;
        imgproc::cvt_color(
            &compare_after,
            &mut graycompare_after,
            imgproc::COLOR_BGR2GRAY,
            0,
        )?;

        core::compare(
            &graycompare_before,
            &graycompare_after,
            &mut diff,
            core::CMP_NE,
        )?;

        // Check if there are any non-zero elements in the diff
        dbg!(&diff);
        dbg!(&compare_before);
        dbg!(&graycompare_before);
        dbg!(&compare_after);
        dbg!(&graycompare_after);

        let non_zero = core::count_non_zero(&diff)?;

        Ok(non_zero > 0)
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
