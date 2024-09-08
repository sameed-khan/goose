use crate::nav::coordinate::PointAsRectAnchor;
use crate::nav::coordinate::{ScreenCoordinates, ScreenRect};
use crate::nav::location::GetLocation;
use crate::nav::location::TargetFactory;
use crate::verb::action::{CheckUIState, GuiAction, GuiVerb};
use autopilot::bitmap::{self, Bitmap};
use autopilot::{
    key,
    key::{Code, KeyCode},
    mouse,
    mouse::Button,
};
use image::GenericImageView;
use std::error::Error;

/// Identifies a textbox by template and inputs a string
/// Parameters:
/// * `target`: Object implementing `LocationStrategy`
/// * `inputString`: Text to input.
/// * `submit`: Optional. Boolean representing whether `Enter` should be pressed after keyboard input. Default false.
/// * `check_zone`: Optional. Rect indicating where to watch for UI state change. Defaults to the
/// rect containing the template match
struct Input {
    target: ScreenCoordinates,
    input_string: String,
    submit: bool,
    check_zone: ScreenRect,
}

impl Input {
    pub fn new(
        target_factory: TargetFactory,
        input_string: String,
        submit: Option<bool>,
        check_zone: Option<ScreenRect>,
    ) -> Self {
        let screen = bitmap::capture_screen().expect("Unable to capture screen");
        screen
            .image
            .save("fixtures/screenshots/init_capture.png")
            .unwrap();
        let target = target_factory.get_location();
        let check_zone = check_zone.unwrap_or_else(|| match &target_factory {
            TargetFactory::AbsoluteTarget(_) => {
                target.generate_rect(150, 150, PointAsRectAnchor::Center)
            }
            TargetFactory::TemplateTarget(template) => {
                let (width, height) = (
                    template.image.width() as f64,
                    template.image.height() as f64,
                );
                let top_left_x = target.x - width / 2.0;
                let top_left_y = target.y - height / 2.0;
                ScreenRect::new(top_left_x, top_left_y, width, height)
            }
        });
        Input {
            target,
            input_string,
            submit: submit.unwrap_or(false),
            check_zone,
        }
    }
}

impl CheckUIState for Input {}

impl GuiAction for Input {
    fn execute(&self) -> Result<Bitmap, Box<dyn Error>> {
        mouse::move_to(self.target.into())?;
        mouse::click(Button::Left, None);
        let screenshot = bitmap::capture_screen()?;
        key::type_string(&self.input_string, &[], 60.0, 0.0);
        if self.submit {
            key::tap(&Code(KeyCode::Return), &[], 100, 0);
        }
        Ok(screenshot)
    }
}

impl GuiVerb for Input {
    /// For Input, check_zone is either custom provided or the area of the template match object
    /// specified.
    fn fire(&self, timeout: Option<u64>) -> Result<(), Box<dyn Error>> {
        let timeout = timeout.unwrap_or(5000);
        self.check_ui_state(timeout, true, None, Some(self.check_zone))?;
        let before = self.execute()?;

        return self.check_ui_state(timeout, false, Some(before), Some(self.check_zone));
    }
}

// NOTE: The tests below are flaky and WILL fail if not run using `cargo test -- --test-threads=1`
#[cfg(test)]
mod tests {
    use super::*;
    use crate::nav::coordinate::Coordinate;
    use crate::nav::location::{AbsoluteLocation, ImageTemplate};
    use crate::nav::strategy::LocationStrategyType;
    use autopilot::{geometry::Point, mouse, screen};
    use rand::prelude::*;
    use std::path::Path;
    use std::process::Command;
    use std::time::Duration;
    use std::{panic, thread};

    const DELAY_BETWEEN_TESTS: u64 = 1;

    trait MockGuiVerb {
        fn fire(&self, timeout: Option<u64>, test_identifier: &str) -> Result<(), Box<dyn Error>>;
    }
    impl MockGuiVerb for Input {
        fn fire(&self, timeout: Option<u64>, test_identifier: &str) -> Result<(), Box<dyn Error>> {
            let timeout = timeout.unwrap_or(5000);

            dbg!(self.check_zone);

            self.check_ui_state(timeout, true, None, Some(self.check_zone))?;

            let before = self.execute()?;

            // apply_check_zone_over_screenshot_and_save(
            //     &before,
            //     self.check_zone.into(),
            //     format!("fixtures/screenshots/before_{}", test_identifier).as_str(),
            // );

            return self.check_ui_state(timeout, false, Some(before), Some(self.check_zone));
        }
    }

    // Setup test fixture to open chrome.exe
    fn setup(program: &str, load_delay: Option<u64>) -> () {
        let load_delay = load_delay.unwrap_or(1);
        let status = Command::new("cmd")
            .args(&["/C", "start", "/max", "", program])
            .status()
            .expect(format!("Failed to open {}", program).as_str());

        if !status.success() {
            panic!("Failed to open program");
        }
        thread::sleep(Duration::from_secs(load_delay));
    }

    fn teardown(program: &str) -> () {
        Command::new("taskkill")
            .args(&["/IM", program, "/F"])
            .spawn()
            .expect(format!("Failed to close {}", program).as_str());

        // Reset mouse to random position on screen
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..screen::size().width as i32);
        let y = rng.gen_range(0..screen::size().height as i32);
        mouse::move_to(Point::new(x as f64, y as f64)).expect("Test: failed mouse move teardown");
    }

    #[test]
    fn input_by_coordinate() {
        setup("notepad.exe", None);
        let input = Input::new(
            TargetFactory::AbsoluteTarget(AbsoluteLocation {
                x: Coordinate::new(25.0),
                y: Coordinate::new(100.0),
            }),
            "Hello, World!".to_string(),
            None,
            None,
        );
        let result = panic::catch_unwind(|| {
            MockGuiVerb::fire(&input, None, "input_by_coordinates").expect("Failed to input text")
        });
        thread::sleep(Duration::from_secs(DELAY_BETWEEN_TESTS));
        teardown("notepad.exe");
        assert!(result.is_ok());
    }

    #[test]
    fn input_by_template() {
        setup("msedge.exe", Some(2));
        let input = Input::new(
            TargetFactory::TemplateTarget(ImageTemplate::new(
                "msedge_omnibox".to_string(),
                Path::new("fixtures/unit/msedge_omnibox.png"),
                None,
                LocationStrategyType::TemplateMatching,
            )),
            "foo".to_string(),
            Some(false),
            None,
        );
        let result = panic::catch_unwind(|| {
            MockGuiVerb::fire(&input, None, "input_by_template").expect("Failed to input text")
        });
        thread::sleep(Duration::from_secs(DELAY_BETWEEN_TESTS));
        teardown("msedge.exe");
        assert!(result.is_ok());
    }

    // Wouldn't parameterization be lovely?
    #[test]
    fn submit_input_by_template() {
        setup("msedge.exe", Some(2));
        let input = Input::new(
            TargetFactory::TemplateTarget(ImageTemplate::new(
                "msedge_omnibox".to_string(),
                Path::new("fixtures/unit/msedge_omnibox.png"),
                None,
                LocationStrategyType::TemplateMatching,
            )),
            "foo".to_string(),
            Some(true),
            None,
        );

        let result = panic::catch_unwind(|| {
            MockGuiVerb::fire(&input, None, "submit_input_by_template")
                .expect("Failed to input text")
        });
        thread::sleep(Duration::from_secs(DELAY_BETWEEN_TESTS));
        teardown("msedge.exe");
        assert!(result.is_ok());
    }
}
