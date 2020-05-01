/// Defines the Skin structure with its default value.
///
/// A skin is a collection of termimad compound_style
use {
    crossterm::style::{
        Attribute::*,
        Attributes,
        Color::AnsiValue,
        Color::{self, *},
    },
    std::{collections::HashMap, fmt},
    termimad::CompoundStyle,
};

macro_rules! Skin {
    (
        $($name:ident: $fg:expr, $bg:expr; $({$a:expr})*)*
    ) => {
        pub struct Skin {
            $(pub $name: CompoundStyle,)*
        }
        impl Skin {
            /// build a skin without any terminal control character (for file output)
            pub fn no_term() -> Skin {
                Skin {
                    $($name: CompoundStyle::default(),)*
                }
            }
            /// build a skin with some entry overloaded by configuration
            pub fn create(mut skin_conf: HashMap<String, CompoundStyle>) -> Skin {
                let mut skin = Skin {
                    $($name: skin_conf.remove(stringify!($name)).unwrap_or(CompoundStyle::new(
                        $fg,
                        $bg,
                        Attributes::from(vec![$($a),*].as_slice()),
                    )),)*
                };
                $(
                    let mut base = skin.default.clone();
                    base.overwrite_with(&skin.$name);
                    skin.$name = base;
                )*
                skin
            }
        }
        impl Clone for Skin {
            fn clone(&self) -> Self {
                Skin {
                    $($name: self.$name.clone(),)*
                }
            }
        }
    }
}

pub fn gray(level: u8) -> Option<Color> {
    Some(AnsiValue(0xE8 + level))
}

pub fn rgb(r: u8, g: u8, b: u8) -> Option<Color> {
    Some(Rgb { r, g, b })
}

pub fn ansi(v: u8) -> Option<Color> {
    Some(AnsiValue(v))
}

Skin! {
    default: gray(22), gray(1);
    tree: gray(5), None;
    file: gray(18), None;
    directory: ansi(110), None; {Bold}
    exe: Some(Cyan), None;
    link: Some(Magenta), None;
    pruning: gray(12), None; {Italic}
    perm__: gray(5), None;
    perm_r: ansi(94), None;
    perm_w: ansi(132), None;
    perm_x: ansi(65), None;
    owner: ansi(138), None;
    group: ansi(131), None;
    dates: ansi(66), None;
    sparse: ansi(214), None;
    git_branch: ansi(178), None;
    git_insertions: ansi(28), None;
    git_deletions: ansi(160), None;
    git_status_current: gray(5), None;
    git_status_modified: ansi(28), None;
    git_status_new: ansi(94), None; {Bold}
    git_status_ignored: gray(17), None;
    git_status_conflicted: ansi(88), None;
    git_status_other: ansi(88), None;
    selected_line: None, gray(4);
    char_match: Some(Green), None;
    file_error: Some(Red), None;
    flag_label: gray(15), None;
    flag_value: ansi(178), None; {Bold}
    input: Some(White), None;
    status_error: gray(22), ansi(124);
    status_job: ansi(220), gray(5);
    status_normal: gray(20), gray(3);
    status_italic: ansi(178), gray(3);
    status_bold: ansi(178), gray(3); {Bold}
    status_code: ansi(229), gray(3);
    status_ellipsis: gray(19), gray(1);
    scrollbar_track: gray(7), None;
    scrollbar_thumb: gray(22), None;
    help_paragraph: gray(20), None;
    help_bold: ansi(178), None; {Bold}
    help_italic: ansi(229), None;
    help_code: gray(21), gray(3);
    help_headers: ansi(178), None;
    help_table_border: ansi(239), None;
}

impl fmt::Debug for Skin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Skin")
    }
}
