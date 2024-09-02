use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

macro_rules! define_error {
    ($error_name:ident) => {
        #[derive(Debug)]
        pub struct $error_name {
            pub message: String,
        }

        impl Display for $error_name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}: {}", stringify!($error_name), self.message)
            }
        }

        impl Error for $error_name {}
    };
}

define_error!(ScreenCoordinateError);
define_error!(OutOfBoundsError);
define_error!(UIActionTimeOutError);
