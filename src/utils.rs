use autopilot::{bitmap::Bitmap, geometry, screen};
use opencv::core::{Mat, Scalar, Vector, CV_8UC3};
use opencv::{core, imgcodecs, imgproc, prelude::*};

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
/// Converts autopilot rect (uses scaled coordinates) to opencv rect (uses physical coordinates)
pub fn convert_aprect_to_ocvrect(rect: geometry::Rect) -> core::Rect {
    let scale = screen::scale();
    core::Rect::new(
        (rect.origin.x * scale) as i32,
        (rect.origin.y * scale) as i32,
        (rect.size.width * scale) as i32,
        (rect.size.height * scale) as i32,
    )
}

pub fn convert_ocvrect_to_aprect(rect: core::Rect) -> geometry::Rect {
    let scale = screen::scale();
    geometry::Rect::new(
        geometry::Point::new(rect.x as f64 / scale, rect.y as f64 / scale),
        geometry::Size::new(rect.width as f64 / scale, rect.height as f64 / scale),
    )
}

pub fn apply_check_zone_over_screenshot_and_save(
    screenshot: &Bitmap,
    check_zone: geometry::Rect,
    filename: &str,
) -> () {
    let mut screenshot_mat = convert_bitmap_to_mat(screenshot);
    let draw_zone = convert_aprect_to_ocvrect(check_zone);
    let _ = imgproc::rectangle(
        &mut screenshot_mat,
        draw_zone,
        Scalar::new(0., 0., 255., 0.),
        1,
        8,
        0,
    );
    imgcodecs::imwrite(
        &format!("{}.png", filename),
        &screenshot_mat,
        &Vector::new(),
    )
    .expect("Failed to write image");
}

pub fn generate_template_match_colormap(
    input_image: &Mat,
    match_result: &Mat,
    template_size: core::Size,
    output_path: &str,
) -> opencv::Result<()> {
    // Normalize match_result to 0-1 range
    let mut normalized_result = Mat::default();
    core::normalize(
        match_result,
        &mut normalized_result,
        0.0,
        255.0,
        core::NORM_MINMAX,
        core::CV_8UC1,
        &core::no_array(),
    )?;

    // Create a color map of the normalized result
    let mut color_map = Mat::default();
    imgproc::apply_color_map(&normalized_result, &mut color_map, imgproc::COLORMAP_JET)?;

    // Create a full-size matrix to hold the color map data
    let mut full_size_color_map = Mat::new_rows_cols_with_default(
        input_image.rows(),
        input_image.cols(),
        color_map.typ(),
        core::Scalar::all(0.0),
    )?;

    // Calculate the correct position to place the color map
    let roi = core::Rect::new(0, 0, match_result.cols(), match_result.rows());

    // Copy the color map into the correct position in the full-size matrix
    color_map.copy_to(&mut full_size_color_map.row_mut(roi.y)?.col_mut(roi.x)?)?;

    // Blend input_image with full_size_color_map
    let mut output = Mat::default();
    core::add_weighted(
        input_image,
        0.7,
        &full_size_color_map,
        0.3,
        0.0,
        &mut output,
        -1,
    )?;

    // Find the location of maximum match
    let mut min_val = 0.0;
    let mut max_val = 0.0;
    let mut min_loc = core::Point::default();
    let mut max_loc = core::Point::default();
    core::min_max_loc(
        match_result,
        Some(&mut min_val),
        Some(&mut max_val),
        Some(&mut min_loc),
        Some(&mut max_loc),
        &core::no_array(),
    )?;

    // Draw rectangle around the best match
    let top_left = max_loc;
    imgproc::rectangle(
        &mut output,
        core::Rect::new(
            top_left.x,
            top_left.y,
            template_size.width,
            template_size.height,
        ),
        core::Scalar::new(0.0, 255.0, 0.0, 0.0),
        2,
        imgproc::LINE_8,
        0,
    )?;

    // Save the output image
    imgcodecs::imwrite(output_path, &output, &Vector::new())?;

    Ok(())
}
