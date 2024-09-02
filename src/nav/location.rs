//! Traits and types for GUI navigation.
use autopilot::screen;
use opencv::{
    core::{min_max_loc, no_array, Mat, Point, Rect, Scalar, CV_8U},
    imgcodecs, imgproc,
    prelude::*,
};
use std::error::Error;
use std::fmt::Debug;
use std::path::Path;
use std::result::Result;

use crate::errors::ScreenCoordinateError;

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
    fn get_location(&self, screenshot: &Mat) -> Result<ScreenCoordinates, Box<dyn Error>>;
}

/// A simple struct to hold information about an image template, used as a basis for navigating the
/// cursor to specific elements and item on-screen.
/// Users define a search region and capture an image of a specific GUI element to match on for cursor
/// movement.
#[derive(Debug)]
pub struct ImageTemplate<'a> {
    pub name: String,                        // Name of the template
    pub path: &'a Path,                      // Path to the image file
    pub search_region: (i32, i32, i32, i32), // top left x, top left y, width, height
}

impl<'a> ImageTemplate<'a> {
    pub fn new(name: String, path: &Path, search_region: (i32, i32, i32, i32)) -> ImageTemplate {
        ImageTemplate {
            name,
            path,
            search_region,
        }
    }
}

impl<'a> LocationStrategy for ImageTemplate<'a> {
    fn get_location(&self, screenshot: &Mat) -> Result<ScreenCoordinates, Box<dyn Error>> {
        let (x, y, width, height) = self.search_region;
        let search_region = Rect::new(x, y, width, height);

        // Create a region of interest (ROI) from the screenshot
        let roi = Mat::roi(screenshot, search_region)?;

        let template = imgcodecs::imread(
            self.path.to_str().unwrap(),
            imgcodecs::ImreadModes::IMREAD_COLOR.into(),
        )?;

        let mut match_result = Mat::default();
        imgproc::match_template(
            &roi,
            &template,
            &mut match_result,
            imgproc::TM_CCOEFF_NORMED,
            &Mat::default(),
        )?;

        let mut match_location = Point::default();
        opencv::core::min_max_loc(
            &match_result,
            None,
            None,
            None,
            Some(&mut match_location),
            &Mat::default(),
        )?;

        // Calculate the absolute coordinates
        let absolute_x = x + match_location.x;
        let absolute_y = y + match_location.y;

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
    fn get_location(&self, _screenshot: &Mat) -> Result<ScreenCoordinates, Box<dyn Error>> {
        Ok(ScreenCoordinates::new(self.x, self.y)?)
    }
}
