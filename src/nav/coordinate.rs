//! Details and behavior of coordinate, which is a core struct in the application.

use crate::errors::ScreenCoordinateError;
use autopilot::geometry;
use autopilot::screen;
use opencv::core;
use std::cmp::min;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
#[derive(Debug, Clone, Copy)]
pub struct Coordinate {
    pub val: f64,
}

impl Coordinate {
    pub fn new<T: Into<f64>>(val: T) -> Self {
        let val = val.into();
        if val > 0.0 {
            Coordinate {
                val: val / screen::scale(),
            }
        } else {
            Coordinate { val: 0.0 }
        }
    }
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.val)
    }
}

impl<T> From<T> for Coordinate
where
    T: Into<f64>,
{
    fn from(val: T) -> Self {
        Coordinate::new(val)
    }
}

/// Defines target location for cursor navigation.
/// Encodes constraint that movement targets must be within the bounds of the screen.
#[derive(Debug, Clone, Copy)]
pub struct ScreenCoordinates {
    pub point: geometry::Point,
}

pub enum PointAsRectAnchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

impl ScreenCoordinates {
    pub fn new<T>(x: T, y: T) -> Result<Self, ScreenCoordinateError>
    where
        T: Into<Coordinate>,
    {
        let coord_x: Coordinate = x.into();
        let coord_y: Coordinate = y.into();
        let screen_size = screen::size(); // returns scaled coordinates not physical
        let width = screen_size.width;
        let height = screen_size.height;

        // Ultimately, the reason negative values or values outside of u16 bounds are not allowed is
        // because that would be outside the screen boundaries

        if coord_x.val > width || coord_y.val > height {
            return Err(ScreenCoordinateError {
                message: format!(
                    "Screen coordinate out of bounds: x: {}, y: {}, screen width: {}, screen height: {}",
                    coord_x, coord_y, width, height
                ),
            });
        }
        Ok(ScreenCoordinates {
            point: geometry::Point::new(coord_x.val, coord_y.val),
        })
    }
    /// Adds the value of x and y to the current coordinates.
    /// Returns error if new coordinates would be out of screen bounds
    pub fn shift<T>(&self, x: T, y: T) -> Result<Self, ScreenCoordinateError>
    where
        T: Into<f64>,
    {
        let new_x = self.point.x + x.into();
        let new_y = self.point.y + y.into();
        ScreenCoordinates::new(new_x, new_y).map_err(|_| ScreenCoordinateError {
            message: format!(
                "Shifted screen coordinates: {:?} are out of bounds",
                (new_x, new_y)
            ),
        })
    }

    /// Generate a rectangle with the screen coordinates as an anchor point. Respects screen bounds.
    /// Sides of the rectangle will be truncated if they exceed the screen boundaries.
    /// Parameters:
    /// * `width`: The width of the rectangle.
    /// * `height`: The height of the rectangle.
    /// * `anchor`: Which anchor point of the rectangle this object represents, can be one of the four points or the center.
    pub fn generate_rect(
        &self,
        width: u64, // TODO: consider after user draw input whether these need to be scaled
        height: u64,
        anchor: PointAsRectAnchor,
    ) -> ScreenRect {
        let (x, y, width, height) = (
            self.point.x as i32,
            self.point.y as i32,
            width as i32,
            height as i32,
        );
        let (max_width, max_height) = (screen::size().width as i32, screen::size().height as i32);

        let (rx, ry, rw, rh) = match anchor {
            PointAsRectAnchor::TopLeft => {
                (x, y, min(width, max_width - x), min(height, max_height - y))
            }
            PointAsRectAnchor::TopRight => {
                let rw = min(width, x);
                let rh = min(height, max_height - y);
                (x - rw, y, rw, rh)
            }
            PointAsRectAnchor::BottomLeft => {
                let rw = min(width, max_width - x);
                let rh = min(height, y);
                (x, y - rh, rw, rh)
            }
            PointAsRectAnchor::BottomRight => {
                let rw = min(width, x);
                let rh = min(height, y);
                (x - rw, y - rh, rw, rh)
            }
            PointAsRectAnchor::Center => {
                let rw = min(width, min(x, max_width - x));
                let rh = min(height, min(y, max_height - y));
                (x - rw / 2, y - rh / 2, rw, rh)
            }
        };
        ScreenRect::new(rx, ry, rw as f64, rh as f64)
    }
}

impl Deref for ScreenCoordinates {
    type Target = geometry::Point;

    fn deref(&self) -> &Self::Target {
        &self.point
    }
}

impl Display for ScreenCoordinates {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self.point)
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

impl From<autopilot::geometry::Point> for ScreenCoordinates {
    fn from(point: autopilot::geometry::Point) -> Self {
        return ScreenCoordinates::new(point.x, point.y).expect("Invalid screen coordinates");
    }
}

/// Defines a rectangle on the screen.
/// Encodes the constraint that the rectangle must be within the bounds of the screen.
#[derive(Clone, Copy)]
pub struct ScreenRect {
    pub rect: geometry::Rect,
}

impl ScreenRect {
    pub fn new<T>(x: T, y: T, width: f64, height: f64) -> Self
    where
        T: Into<Coordinate>,
    {
        let coord_x: Coordinate = x.into();
        let coord_y: Coordinate = y.into();
        let width = min(
            width as u64,
            screen::size().width as u64 - coord_x.val as u64,
        ) as f64;
        let height = min(
            height as u64,
            screen::size().height as u64 - coord_y.val as u64,
        ) as f64;

        ScreenRect {
            rect: geometry::Rect::new(
                geometry::Point::new(coord_x.val, coord_y.val),
                autopilot::geometry::Size::new(width, height),
            ),
        }
    }
}

impl Default for ScreenRect {
    fn default() -> Self {
        ScreenRect::new(0, 0, screen::size().width, screen::size().height)
    }
}

impl From<geometry::Rect> for ScreenRect {
    fn from(rect: geometry::Rect) -> Self {
        ScreenRect::new(
            rect.origin.x,
            rect.origin.y,
            rect.size.width,
            rect.size.height,
        )
    }
}

impl From<ScreenRect> for geometry::Rect {
    fn from(screen_rect: ScreenRect) -> Self {
        screen_rect.rect
    }
}

impl From<ScreenRect> for core::Rect {
    fn from(screen_rect: ScreenRect) -> Self {
        core::Rect::new(
            screen_rect.rect.origin.x as i32,
            screen_rect.rect.origin.y as i32,
            screen_rect.rect.size.width as i32,
            screen_rect.rect.size.height as i32,
        )
    }
}

impl From<core::Rect> for ScreenRect {
    fn from(rect: core::Rect) -> Self {
        ScreenRect::new(rect.x, rect.y, rect.width as f64, rect.height as f64)
    }
}

impl Display for ScreenRect {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self.rect)
    }
}

impl Debug for ScreenRect {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self.rect)
    }
}
