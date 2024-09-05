//! Traits and types for GUI navigation.
use autopilot::bitmap::Bitmap;
use autopilot::screen;
use opencv::{
    core,
    core::{min_max_loc, no_array, Mat, Point, Rect, CV_8UC3},
    imgcodecs, imgproc,
    imgproc::match_template,
    prelude::*,
};
use std::error::Error;
use std::fmt::Debug;
use std::path::Path;
use std::result::Result;

use crate::errors::ScreenCoordinateError;

/// Link between Bitmap and Mat types to allow for OpenCV template matching from autopilot screengrab
/// Bitmaps.
pub fn convert_bitmap_to_mat(screen: &Bitmap) -> Mat {
    let width = screen.size.width as i32;
    let height = screen.size.height as i32;
    let raw_pixels = screen.image.raw_pixels();

    // Create a Mat from the raw pixels
    let bgr_mat = unsafe {
        Mat::new_rows_cols_with_data_unsafe(
            height,
            width,
            CV_8UC3,
            raw_pixels.as_ptr() as *mut std::ffi::c_void,
            core::Mat_AUTO_STEP,
        )
        .expect("Failed to create Mat from raw pixels")
    };

    // Convert from RGB to BGR (OpenCV uses BGR by default)
    let mut opencv_mat = Mat::default();
    imgproc::cvt_color(&bgr_mat, &mut opencv_mat, imgproc::COLOR_RGB2BGR, 0)
        .expect("Failed to convert color");

    opencv_mat
}

/// Defines target location for cursor navigation.
/// Encodes constraint that movement targets must be within the bounds of the screen.
pub struct ScreenCoordinates {
    pub x: u16,
    pub y: u16,
}

impl ScreenCoordinates {
    pub fn new<T>(x: T, y: T) -> Result<Self, ScreenCoordinateError>
    where
        T: TryInto<u16> + Copy,
        <T as TryInto<u16>>::Error: Debug,
    {
        let screen_size = screen::size();
        let width = screen_size.width as u16;
        let height = screen_size.height as u16;

        // Ultimately, the reason negative values or values outside of u16 bounds are not allowed is
        // because that would be outside the screen boundaries
        let x = x.try_into().map_err(|e| ScreenCoordinateError {
            message: format!("Screen coordinate x out of bounds: {:?}", e),
        })?;
        let y = y.try_into().map_err(|e| ScreenCoordinateError {
            message: format!("Screen coordinate y out of bounds: {:?}", e),
        })?;

        if x > width || y > height {
            return Err(ScreenCoordinateError {
                message: format!(
                    "Screen coordinate out of bounds: x: {}, y: {}, width: {}, height: {}",
                    x, y, width, height
                ),
            });
        }
        Ok(ScreenCoordinates { x, y })
    }
}

impl From<ScreenCoordinates> for autopilot::geometry::Point {
    fn from(screen_coordinates: ScreenCoordinates) -> Self {
        autopilot::geometry::Point {
            x: screen_coordinates.x as f64,
            y: screen_coordinates.y as f64,
        }
    }
}

pub trait LocationStrategy {
    fn get_location(&self, screenshot: &Bitmap) -> Result<ScreenCoordinates, Box<dyn Error>>;
}

/// A simple struct to hold information about an image template, used as a basis for navigating the
/// cursor to specific elements and item on-screen.
/// Users define a search region and capture an image of a specific GUI element to match on for cursor
/// movement.
/// Note: The search region is defined to provide specificity for multiple occurrence of the same
/// GUI element on the screen. However, the algorithm performs best for matching when given the
/// entire screen as the search region.
#[derive(Debug)]
pub struct ImageTemplate<'a> {
    pub name: String,                        // Name of the template
    pub path: &'a Path,                      // Path to the image file
    pub search_region: (i32, i32, i32, i32), // top left x, top left y, width, height
}

impl<'a> ImageTemplate<'a> {
    pub fn new(
        name: String,
        path: &Path,
        search_region: Option<(i32, i32, i32, i32)>,
    ) -> ImageTemplate {
        let search_region = search_region.unwrap_or((
            0,
            0,
            screen::size().width as i32,
            screen::size().height as i32,
        ));
        ImageTemplate {
            name,
            path,
            search_region,
        }
    }
}

impl<'a> LocationStrategy for ImageTemplate<'a> {
    fn get_location(&self, screenshot_bmp: &Bitmap) -> Result<ScreenCoordinates, Box<dyn Error>> {
        let (x, y, width, height) = self.search_region;
        let search_region = Rect::new(x, y, width, height);

        let screenshot = convert_bitmap_to_mat(screenshot_bmp);
        // Create a region of interest (ROI) from the screenshot
        let roi = Mat::roi(&screenshot, search_region)?;
        let template = imgcodecs::imread(
            self.path.to_str().unwrap(),
            imgcodecs::ImreadModes::IMREAD_COLOR.into(),
        )?;

        let mut match_result = Mat::default();
        match_template(
            &roi,
            &template,
            &mut match_result,
            imgproc::TM_CCOEFF_NORMED,
            &no_array(),
        )?;

        let mut match_location = Point::default();
        min_max_loc(
            &match_result,
            None,
            None,
            None,
            Some(&mut match_location),
            &no_array(),
        )?;

        // Calculate the absolute coordinates
        let template_width = template.size()?.width;
        let template_height = template.size()?.height;

        // // Template match seems to return the top left corner of the match
        // // so add half the width and height to get the center
        let absolute_x = x + match_location.x + template_width / 2;
        let absolute_y = y + match_location.y + template_height / 2;

        let result = ScreenCoordinates::new(absolute_x, absolute_y)?;

        Ok(result)
    }
}

#[derive(Debug)]
pub struct AbsoluteLocation {
    pub x: u16,
    pub y: u16,
}

impl LocationStrategy for AbsoluteLocation {
    fn get_location(&self, _screenshot: &Bitmap) -> Result<ScreenCoordinates, Box<dyn Error>> {
        Ok(ScreenCoordinates::new(self.x, self.y)?)
    }
}
