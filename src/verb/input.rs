use crate::errors::UIActionTimeOutError;
use crate::nav::location::convert_bitmap_to_mat;
use crate::nav::location::LocationStrategy;
use crate::verb::action::{CheckUIState, GuiAction, GuiVerb};
use autopilot::bitmap::Bitmap;
use autopilot::{
    geometry::{Point, Rect, Size},
    key,
    key::{Code, KeyCode},
    mouse,
    mouse::Button,
};
use std::error::Error;
use std::{thread, time};

/// Identifies a textbox by template and inputs a string
/// Parameters:
/// * `target`: Object implementing `LocationStrategy`
/// * `inputString`: Text to input.
/// * `submit`: Optional. Boolean representing whether `Enter` should be pressed after keyboard input. Default false.
/// * `check_zone`: Optional. Rect indicating where to watch for UI state change. Defaults to the
/// rect containing the template match
struct Input<L: LocationStrategy> {
    target: L,
    input_string: String,
    submit: bool,
    check_zone: Option<Rect>,
}

impl<L: LocationStrategy> Input<L> {
    pub fn new(
        target: L,
        input_string: String,
        submit: Option<bool>,
        check_zone: Option<Rect>,
    ) -> Self {
        Input {
            target,
            input_string,
            submit: submit.unwrap_or(false),
            check_zone,
        }
    }
}

impl<L: LocationStrategy> CheckUIState for Input<L> {}

impl<L: LocationStrategy> GuiAction for Input<L> {
    fn execute(&self) -> Result<Bitmap, Box<dyn Error>> {
        let tmp = self.get_screenshot()?;
        let location: Point = (self.target.get_location(&tmp)?).into();

        mouse::move_to(location)?;
        mouse::click(Button::Left, None);
        let screenshot = self.get_screenshot()?;
        key::type_string(&self.input_string, &[], 60.0, 0.0);
        if self.submit {
            key::tap(&Code(KeyCode::Return), &[], 100, 0);
        }
        Ok(screenshot)
    }
}

impl<L: LocationStrategy> GuiVerb for Input<L> {
    /// For Input, check_zone is either custom provided or the area of the template match object
    /// specified.
    fn get_check_zone(&self) -> Rect {
        match self.check_zone {
            Some(rect) => rect,
            None => {
                let tmp = self.get_screenshot().expect("Unable to screenshot");
                let template_location: Point = (self
                    .target
                    .get_location(&tmp)
                    .expect("Unable to screenshot"))
                .into();
                let rect_size = Size::new(200.0, 200.0);
                Rect::new(template_location, rect_size)
            }
        }
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
