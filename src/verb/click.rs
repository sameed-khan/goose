use crate::errors::UIActionTimeOutError;
use crate::nav::coordinate::PointAsRectAnchor;
use crate::nav::coordinate::ScreenCoordinates;
use crate::nav::coordinate::ScreenRect;
use crate::nav::location::{GetLocation, TargetFactory};
use crate::nav::strategy::{LocationStrategy, LocationStrategyType};
use crate::verb::action::{CheckUIState, GuiAction, GuiVerb};
use autopilot::bitmap::{self, Bitmap};
use autopilot::geometry::Point;
use autopilot::{mouse, mouse::Button};
use image::GenericImageView;
use opencv::prelude::*;
use std::error::Error;
use std::time::{Duration, Instant};

/// Clicks the mouse at the given location.
struct Click {
    target: ScreenCoordinates,
    button: Button,
    check_zone: ScreenRect,
}
impl Click {
    pub fn new(
        target_factory: TargetFactory,
        button: Button,
        check_zone: Option<ScreenRect>,
    ) -> Self {
        let screenshot = bitmap::capture_screen().expect("Unable to capture screen");
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
                ScreenRect::new(target.x, target.y, width, height)
            }
        });
        Click {
            target,
            button,
            check_zone,
        }
    }
}

impl CheckUIState for Click {}

impl GuiAction for Click {
    fn execute(&self) -> Result<Bitmap, Box<dyn Error>> {
        let location: Point = self.target.into();

        mouse::move_to(location)?;
        let screenshot = bitmap::capture_screen_portion(self.check_zone.into())?;
        mouse::click(self.button, None);
        Ok(screenshot)
    }
}

impl GuiVerb for Click {
    fn fire(&self, timeout: Option<u64>) -> Result<(), Box<dyn Error>> {
        let timeout = timeout.unwrap_or(500);
        self.check_ui_state(timeout, true, None, Some(self.check_zone))?;
        let before = self.execute()?;

        return self.check_ui_state(timeout, false, Some(before), Some(self.check_zone));
    }
}

// NOTE: The tests below are flaky and WILL fail if not run using `cargo test -- --test-threads=1`
#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::UIActionTimeOutError;
    use crate::nav::coordinate::Coordinate;
    use crate::nav::location::{AbsoluteLocation, ImageTemplate};
    use std::path::Path;
    use std::process::Command;
    use std::{thread, time};

    const DELAY_BETWEEN_TESTS: u64 = 2;

    // Setup test fixture to open notepad.exe
    fn setup() -> () {
        Command::new("cmd")
            .args(&["/C", "start", "", "/max", "notepad.exe"])
            .spawn()
            .expect("Failed to open notepad.exe");

        thread::sleep(time::Duration::from_secs(1));
    }

    fn teardown() -> () {
        Command::new("taskkill")
            .args(&["/IM", "notepad.exe", "/F"])
            .spawn()
            .expect("Failed to close notepad.exe");
    }

    #[test]
    // Test Click verb with AbsoluteCoordinates
    fn click_by_coordinates() {
        setup();

        let click = Click::new(
            TargetFactory::AbsoluteTarget(AbsoluteLocation {
                x: Coordinate::new(1890),
                y: Coordinate::new(10),
            }),
            Button::Left,
            None,
        );

        if let Err(e) = click.fire(None) {
            println!("Error: {}", e);
            teardown()
        }

        thread::sleep(time::Duration::from_secs(DELAY_BETWEEN_TESTS));
    }

    #[test]
    fn click_by_template() {
        setup();

        let click = Click::new(
            TargetFactory::TemplateTarget(ImageTemplate::new(
                "notepad_close_button".to_string(),
                Path::new("fixtures/notepad_close_button.png"),
                None,
                LocationStrategyType::TemplateMatching,
            )),
            Button::Left,
            None,
        );

        if let Err(e) = click.fire(None) {
            println!("Error: {}", e);
            teardown()
        }

        thread::sleep(time::Duration::from_secs(DELAY_BETWEEN_TESTS));
    }

    #[test]
    fn no_ui_change_after_click_should_error() {
        setup();
        let click = Click::new(
            TargetFactory::AbsoluteTarget(AbsoluteLocation {
                x: Coordinate::new(500),
                y: Coordinate::new(500),
            }),
            Button::Left,
            None,
        );

        let click_err = click.fire(None).unwrap_err();
        let downcast_err = click_err.downcast_ref::<UIActionTimeOutError>();
        assert!(downcast_err.is_some());

        teardown();

        thread::sleep(time::Duration::from_secs(DELAY_BETWEEN_TESTS));
    }
}
