
use std::collections::HashMap;
use termion::color::{self, *};

macro_rules! Skin {
    (
        $($name:ident: $fg:expr, $bg:expr)*
    ) => {
        pub struct Skin {
            $(pub $name: String,)*
            pub reset: String,
        }
        impl Skin {
            pub fn create(mut skin_conf: HashMap<String, String>) -> Skin {
                Skin {
                    $($name: skin_conf.remove(stringify!($name)).unwrap_or(
                        format!("{}{}", color::Fg($fg), color::Bg($bg))
                    ),)*
                    reset: format!("{}{}", color::Bg(color::Reset), color::Fg(color::Reset)),
                }
            }
        }
    }
}

Skin! {
    status_normal: White, AnsiValue::grayscale(2)
    status_error: Red, AnsiValue::grayscale(2)
}

impl Skin {
    pub fn make() -> Skin {
        let mut map = HashMap::new();
        map.insert(
            "status_normal".to_string(),
            format!(
                "{}{}",
                color::Fg(color::Yellow),
                color::Bg(color::AnsiValue::grayscale(8))
            )
        );
        Skin::create(map)
    }
}

