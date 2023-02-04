use {
    crate::{
        app::{AppContext, DisplayContext},
        command::ScrollCommand,
        display::{DisplayableTree, Screen, W},
        errors::ProgramError,
        pattern::InputPattern,
        skin::PanelSkin,
        task_sync::Dam,
        tree_build::{TreeBuilder},
        tree::{Tree, TreeOptions},
    },
    crokey::crossterm::{
        cursor,
        QueueableCommand,
    },
    std::{
        io,
        path::PathBuf,
    },
    termimad::Area,
};

pub struct DirView {
    pub tree: Tree,
    page_height: Option<usize>,
}
impl DirView {
    pub fn new(
        dir: PathBuf,
        pattern: InputPattern,
        dam: &Dam,
        con: &AppContext,
    ) -> Result<Self, io::Error> {
        let options = TreeOptions {
            show_hidden: true,
            respect_git_ignore: false,
            pattern,
            ..Default::default()
        };
        let mut builder = TreeBuilder::from(
            dir,
            options,
            100,
            con,
        ).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        builder.deep = false;
        let tree = builder
            .build_tree(
                false, // on refresh we always do a non total search
                dam,
            )
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(Self {
            tree,
            page_height: None,
        })
    }
    pub fn display(
        &mut self,
        w: &mut W,
        disc: &DisplayContext,
        area: &Area,
    ) -> Result<(), ProgramError> {
        let page_height = area.height as usize;
        if Some(page_height) != self.page_height {
            self.page_height = Some(page_height);
        }
        let dp = DisplayableTree {
            app_state: None,
            tree: &self.tree,
            skin: &disc.panel_skin.styles,
            ext_colors: &disc.con.ext_colors,
            area: area.clone(),
            in_app: true,
        };
        dp.write_on(w)?;
        Ok(())
    }
    pub fn display_info(
        &mut self,
        w: &mut W,
        _screen: Screen,
        panel_skin: &PanelSkin,
        area: &Area,
    ) -> Result<(), ProgramError> {
        let width = area.width as usize;
        let mut s = format!("{}", self.tree.lines.len());
        if s.len() > width {
            return Ok(());
        }
        if s.len() + "lines: ".len() < width {
            s = format!("entries: {s}");
        }
        w.queue(cursor::MoveTo(
            area.left + area.width - s.len() as u16,
            area.top,
        ))?;
        panel_skin.styles.default.queue(w, s)?;
        Ok(())
    }
    pub fn try_scroll(
        &mut self,
        cmd: ScrollCommand,
    ) -> bool {
        let Some(page_height) = self.page_height else {
            return false;
        };
        let dy = cmd.to_lines(page_height);
        self.tree.try_scroll(dy, page_height)
    }
    pub fn try_select_y(&mut self, y: u16) -> bool {
        self.tree.try_select_y(y as usize)
    }
    pub fn move_selection(&mut self, dy: i32, cycle: bool) {
        if let Some(page_height) = self.page_height {
            self.tree.move_selection(dy, page_height, cycle);
        }
    }
    pub fn select_first(&mut self) {
        self.tree.try_select_first();
    }
    pub fn select_last(&mut self) {
        if let Some(page_height) = self.page_height {
            self.tree.try_select_last(page_height);
        }
    }
}
