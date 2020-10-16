//! build the list_view used for displaying the mounts

use {
    super::*,
    crate::{
        app::*,
        command::{Command, TriggerType},
        conf::Conf,
        display::{CropWriter, SPACE_FILLING, Screen, W},
        errors::ProgramError,
        filesystems,
        launchable::Launchable,
        pattern::*,
        skin::PanelSkin,
        verb::*,
    },
    crossterm::{
        cursor,
        style::{Color, Print, SetForegroundColor},
        QueueableCommand,
    },
    lfs_core::{
        self,
        Mount,
    },
    std::{
        path::Path,
    },
    strict::NonEmptyVec,
    termimad::{
        ansi, Alignment, Area, CompoundStyle, ListView, ListViewCell, ListViewColumn, MadSkin,
        ProgressBar,
    },
};

pub fn make_list_view() -> ListView<Mount, PanelSkin> {
    let columns = vec![
        ListViewColumn::new(
            "name",
            10,
            50,
            Box::new(|mount: &Mount, skin: | {
                ListViewCell::new(
                    mount.info.mount_point.to_string_lossy().to_string(),
                    if fi.is_dir {
                        &SKIN.bold
                    } else {
                        &SKIN.paragraph.compound_style
                    },
                )
            }),
        )
        .with_align(Alignment::Left),
    ];
