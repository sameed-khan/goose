use crate::nav::location::{AbsoluteLocation, ImageTemplate, LocationStrategy};
use crate::verb::action::{CheckUIState, GuiAction, GuiVerb};
use autopilot::{geometry::Point, mouse, mouse::Button};
use opencv::core::Mat;
use std::error::Error;
use std::{thread, time};

/// Clicks the mouse at the given location.
struct Click<L: LocationStrategy> {
    target: L,
    button: Button,
}

impl<L: LocationStrategy> Click<L> {
    pub fn new(target: L, button: Button) -> Self {
        Click { target, button }
    }
}

impl<L: LocationStrategy> CheckUIState for Click<L> {}

impl<L: LocationStrategy> GuiAction for Click<L> {
    fn execute(&self) -> Result<Mat, Box<dyn Error>> {
        let location: Point = (self.target.get_location(&(self.get_screenshot()?))?).into();
        mouse::move_to(location)?;
        let screenshot = self.get_screenshot()?; // Take a screenshot after moving the mouse
        println!("Screenshot captured after moving the mouse");
        mouse::click(self.button, None);
        Ok(screenshot)
    }
}

impl<L: LocationStrategy> GuiVerb for Click<L> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    // Setup test fixture to open notepad.exe
    fn setup() -> () {
        Command::new("cmd")
            .args(&["/C", "start", "", "/max", "notepad.exe"])
            .spawn()
            .expect("Failed to open notepad.exe");

        thread::sleep(time::Duration::from_secs(2));
    }

    fn teardown() -> () {
        Command::new("taskkill")
            .args(&["/IM", "notepad.exe", "/F"])
            .spawn()
            .expect("Failed to close notepad.exe");
    }

    #[test]
    // Test Click verb with AbsoluteCoordinates
    fn test_click_coordinates() {
        setup();

        let click = Click::new(AbsoluteLocation { x: 1890, y: 10 }, Button::Left);
        click.fire(None, None).unwrap();

        teardown();
    }

    #[test]
    fn test_click_template() {
        setup();

        let click = Click::new(
            ImageTemplate::new(
                "notepad_close_button".to_string(),
                std::path::Path::new("fixtures/notepad_close_button.png"),
                (1050, 130, 100, 100),
            ),
            Button::Left,
        );
        let current_screenshot = click.get_screenshot().unwrap();
        click.execute().unwrap();
        assert!(click
            .changed_ui_state(&current_screenshot, &click.get_screenshot().unwrap(), None)
            .unwrap());

        teardown();
    }
}
