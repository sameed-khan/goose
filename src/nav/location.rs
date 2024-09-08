//! Traits and types for GUI navigation.
use crate::nav::coordinate::Coordinate;
use crate::nav::coordinate::{ScreenCoordinates, ScreenRect};
use crate::nav::strategy::{
    BitmapNeedleStrategy, EdgeParsingStrategy, LocationStrategy, LocationStrategyType,
    TemplateMatchingStrategy,
};
use autopilot::screen;
use image::GenericImageView;
use image::{io::Reader, DynamicImage};
use std::fmt::Debug;
use std::path::Path;

pub trait GetLocation {
    fn get_location(&self) -> ScreenCoordinates;
}

/// A simple struct to hold information about an image template, used as a basis for navigating the
/// cursor to specific elements and item on-screen.
/// Users define a search region and capture an image of a specific GUI element to match on for cursor
/// movement.
/// Note: The search region is defined to provide specificity for multiple occurrence of the same
/// GUI element on the screen. However, the algorithm performs best for matching when given the
/// entire screen as the search region.
pub struct ImageTemplate {
    pub name: String,
    pub image: DynamicImage,
    pub search_region: (i32, i32, i32, i32), // top left x, top left y, width, height
    pub location_strategy: Box<dyn LocationStrategy>,
}

impl ImageTemplate {
    pub fn new(
        name: String,
        path: &Path,
        search_region: Option<(Coordinate, Coordinate, Coordinate, Coordinate)>,
        strategy_type: LocationStrategyType,
    ) -> ImageTemplate {
        let ssize = screen::size(); // Gets screen size in SCALED coordinates
        let output_sr = match search_region {
            Some(region) => (
                region.0.val as i32,
                region.1.val as i32,
                region.2.val as i32,
                region.3.val as i32,
            ),
            None => (0, 0, ssize.width as i32, ssize.height as i32),
        };
        let image = Reader::open(path)
            .expect("Failed to read image file")
            .decode()
            .expect("Failed to read image file");

        let location_strategy: Box<dyn LocationStrategy> = match strategy_type {
            LocationStrategyType::TemplateMatching => Box::new(TemplateMatchingStrategy {
                template_path: String::from(
                    path.to_str()
                        .expect(&format!("Path {:?} is not valid unicode", path)),
                ),
            }),
            LocationStrategyType::BitmapNeedle => Box::new(BitmapNeedleStrategy {
                template_path: String::from(
                    path.to_str()
                        .expect(&format!("Path {:?} is not valid unicode", path)),
                ),
            }),
            LocationStrategyType::EdgeParsing => Box::new(EdgeParsingStrategy {
                template_path: String::from(
                    path.to_str()
                        .expect(&format!("Path {:?} is not valid unicode", path)),
                ),
            }),
        };
        ImageTemplate {
            name,
            image,
            search_region: output_sr,
            location_strategy,
        }
    }
}

impl GetLocation for ImageTemplate {
    /// Gets target location based on image template and matching strategy
    /// Returns:
    /// * `ScreenCoordinates` - the *_CENTER_* of the image template on screen
    fn get_location(&self) -> ScreenCoordinates {
        let (x, y, width, height) = self.search_region;
        let (x, y, width, height) = (x as f64, y as f64, width as f64, height as f64);
        let screen_coords = self
            .location_strategy
            .get_location(Some(ScreenRect::new(x, y, width, height)))
            .unwrap();

        // Shift the coordinates to the center of the image
        return screen_coords
            .shift(
                self.image.width() as f64 / 2.0,
                self.image.height() as f64 / 2.0,
            )
            .expect("Image template dimensions out of screen bounds");
    }
}

impl Debug for ImageTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageTemplate")
            .field("name", &self.name)
            .field("search_region", &self.search_region)
            .finish()
    }
}

#[derive(Debug)]
pub struct AbsoluteLocation {
    pub x: Coordinate,
    pub y: Coordinate,
}

pub enum TargetFactory {
    TemplateTarget(ImageTemplate),
    AbsoluteTarget(AbsoluteLocation),
}

impl GetLocation for AbsoluteLocation {
    fn get_location(&self) -> ScreenCoordinates {
        ScreenCoordinates::new(self.x.val, self.y.val).unwrap()
    }
}

impl<'a> GetLocation for TargetFactory {
    fn get_location(&self) -> ScreenCoordinates {
        match self {
            TargetFactory::TemplateTarget(template) => template.get_location(),
            TargetFactory::AbsoluteTarget(absolute_location) => absolute_location.get_location(),
        }
    }
}
