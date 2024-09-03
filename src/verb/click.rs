use crate::errors::UIActionTimeOutError;
use crate::nav::location::convert_bitmap_to_mat;
use crate::nav::location::LocationStrategy;
use crate::verb::action::{CheckUIState, GuiAction, GuiVerb};
use autopilot::bitmap::Bitmap;
use autopilot::{
    geometry::{Point, Rect, Size},
    mouse,
    mouse::Button,
};
use std::error::Error;
use std::{thread, time};

/// Clicks the mouse at the given location.
struct Click<L: LocationStrategy> {
    target: L,
    button: Button,
    check_radius: u16,
}

impl<L: LocationStrategy> Click<L> {
    pub fn new(target: L, button: Button, check_radius: u16) -> Self {
        Click {
            target,
            button,
            check_radius,
        }
    }
}

impl<L: LocationStrategy> CheckUIState for Click<L> {}

impl<L: LocationStrategy> GuiAction for Click<L> {
    fn execute(&self) -> Result<Bitmap, Box<dyn Error>> {
        let tmp = self.get_screenshot()?;
        let tmp_mat = convert_bitmap_to_mat(tmp);
        let location: Point = (self.target.get_location(&tmp_mat)?).into();

        mouse::move_to(location)?;
        let screenshot = self.get_screenshot()?; // Take a screenshot after moving the mouse
        println!("Screenshot captured after moving the mouse");
        mouse::click(self.button, None);
        Ok(screenshot)
    }
}

impl<L: LocationStrategy> GuiVerb for Click<L> {
    fn get_check_zone(&self) -> Rect {
        let cursor_location = mouse::location();
        let rect_origin = Point::new(
            cursor_location.x - self.check_radius as f64,
            cursor_location.y - self.check_radius as f64,
        );
        let rect_size = Size::new(
            (self.check_radius * 2) as f64,
            (self.check_radius * 2) as f64,
        );
        Rect::new(rect_origin, rect_size)
    }

    fn fire(&self, wait_duration: Option<u64>, timeout: Option<u64>) -> Result<(), Box<dyn Error>> {
        let mut timeout = timeout.unwrap_or(1000);
        let wait_duration = wait_duration.unwrap_or(100);

        let before = self.execute()?;
        let mut after;

        while timeout > 0 {
            thread::sleep(time::Duration::from_millis(wait_duration));
            after = self.get_screenshot()?;
            if self.changed_ui_state(&before, &mut after, Some(self.get_check_zone()))? {
                return Ok(());
            }
            timeout -= 100;
        }

        return Err(Box::new(UIActionTimeOutError {
            message: "UI action timed out".to_string(),
        }));
    }
}

// NOTE: The tests below are flaky and WILL fail if not run using `cargo test -- --test-threads=1`
#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::UIActionTimeOutError;
    use crate::nav::location::{AbsoluteLocation, ImageTemplate};
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

        let click = Click::new(AbsoluteLocation { x: 1890, y: 10 }, Button::Left, 50);

        if let Err(e) = click.fire(None, None) {
            println!("Error: {}", e);
            teardown()
        }

        thread::sleep(time::Duration::from_secs(DELAY_BETWEEN_TESTS));
    }

    #[test]
    fn click_by_template() {
        setup();

        let click = Click::new(
            ImageTemplate::new(
                "notepad_close_button".to_string(),
                std::path::Path::new("fixtures/notepad_close_button.png"),
                None,
            ),
            Button::Left,
            50,
        );

        if let Err(e) = click.fire(None, None) {
            println!("Error: {}", e);
            teardown()
        }

        thread::sleep(time::Duration::from_secs(DELAY_BETWEEN_TESTS));
    }

    #[test]
    fn no_ui_change_after_click_should_error() {
        setup();
        let click = Click::new(AbsoluteLocation { x: 500, y: 500 }, Button::Left, 50);

        let click_err = click.fire(None, None).unwrap_err();
        let downcast_err = click_err.downcast_ref::<UIActionTimeOutError>();
        assert!(downcast_err.is_some());

        teardown();

        thread::sleep(time::Duration::from_secs(DELAY_BETWEEN_TESTS));
    }
}
