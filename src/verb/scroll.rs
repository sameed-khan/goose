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

/// Scrolls the interface, contains two variants:
/// * `IterativeScroll`: Scrolls an interface until it 'hits the bottom' - has a defined search
/// region at the bottom of the scroll window interface that determines when the scrolling has 
/// completed.
/// * `SeekScroll`: Scrolls an interface until a specific visual element appears - has a predefined
/// image template that it will poll against the screen simultaneously 