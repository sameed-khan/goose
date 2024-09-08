use crate::nav::coordinate::{ScreenCoordinates, ScreenRect};
use crate::utils::convert_bitmap_to_mat;
use autopilot::{
    bitmap::{self, capture_screen, Bitmap},
    geometry, screen,
};
use image::io::Reader;
use opencv::{
    core::{self, min_max_loc, no_array, Mat},
    imgcodecs,
    imgproc::{self, match_template, resize, INTER_AREA},
    prelude::*,
};
use std::error::Error;

pub trait LocationStrategy {
    fn get_location(
        &self,
        search_region: Option<ScreenRect>,
    ) -> Result<ScreenCoordinates, Box<dyn Error>>;
}

pub struct TemplateMatchingStrategy {
    pub template_path: String,
}

pub struct BitmapNeedleStrategy {
    pub template_path: String,
}

pub struct EdgeParsingStrategy {
    pub template_path: String,
}

pub enum LocationStrategyType {
    TemplateMatching,
    BitmapNeedle,
    EdgeParsing,
}

impl LocationStrategy for TemplateMatchingStrategy {
    fn get_location(
        &self,
        search_region: Option<ScreenRect>,
    ) -> Result<ScreenCoordinates, Box<dyn Error>> {
        let screenshot = capture_screen()?;
        let search_region = search_region.unwrap_or(ScreenRect::default());
        let search_region: core::Rect = search_region.into();
        let screenshot = convert_bitmap_to_mat(&screenshot);

        let roi = Mat::roi(&screenshot, search_region)?;

        let template = imgcodecs::imread(&self.template_path, imgcodecs::IMREAD_COLOR)?;
        let mut template_scaled = Mat::default();
        let dst_size = template_scaled.size()?;

        resize(
            &template,
            &mut template_scaled,
            dst_size,
            1.0 / screen::scale(),
            1.0 / screen::scale(),
            INTER_AREA,
        )?;

        let mut match_result = Mat::default();
        match_template(
            &roi,
            &template_scaled,
            &mut match_result,
            imgproc::TM_CCOEFF_NORMED,
            &no_array(),
        )?;

        // dbg!(&roi);
        // dbg!(&template_scaled);
        // dbg!(&match_result);
        // dbg!(self.search_region);
        // generate_template_match_colormap(
        //     &screenshot,
        //     &match_result,
        //     template.size()?,
        //     format!("fixtures/screenshots/{}_match_colormap.png", self.name).as_str(),
        // )?;
        // let mut normalized_result = Mat::default();
        // core::normalize(
        //     &match_result,
        //     &mut normalized_result,
        //     0.0,
        //     255.0,
        //     core::NORM_MINMAX,
        //     core::CV_8U,
        //     &no_array(),
        // )?;
        // imgcodecs::imwrite(
        //     format!("fixtures/screenshots/{}_match_result.png", self.name).as_str(),
        //     &normalized_result,
        //     &Vector::new(),
        // )?;

        let mut match_location = core::Point::default();
        min_max_loc(
            &match_result,
            None,
            None,
            None,
            Some(&mut match_location),
            &no_array(),
        )?;

        // ScreenCoordinates takes any type convertible into Coordinate
        // therefore absolute_x and absolute_y will be silently rescaled to be scaled coordinates
        // instead of physical coordinates
        let result = ScreenCoordinates::new(match_location.x, match_location.y)?;

        Ok(result)
    }
}

impl LocationStrategy for BitmapNeedleStrategy {
    fn get_location(
        &self,
        search_region: Option<ScreenRect>,
    ) -> Result<ScreenCoordinates, Box<dyn Error>> {
        let needle = Bitmap::new(
            Reader::open(&self.template_path)
                .expect("Failed to read image file")
                .decode()
                .expect("Failed to read image file"),
            Some(screen::scale()),
        );

        let screenshot = capture_screen()?;
        let search_region = search_region.unwrap_or(ScreenRect::default());
        let search_region: geometry::Rect = search_region.into();
        let found = screenshot
            .find_bitmap(&needle, Some(0.8), Some(search_region), None)
            .expect("Template not found in image")
            .into();

        Ok(found)
    }
}

impl LocationStrategy for EdgeParsingStrategy {
    fn get_location(
        &self,
        search_region: Option<ScreenRect>,
    ) -> Result<ScreenCoordinates, Box<dyn Error>> {
        unimplemented!();
    }
}
