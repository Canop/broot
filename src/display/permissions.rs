use {
    super::CropWriter,
    crate::{
        display::cond_bg,
        errors::ProgramError,
        permissions,
        skin::StyleMap,
        tree::{Tree, TreeLine},
    },
    std::{
        io::Write,
        os::unix::fs::MetadataExt,
    },
    umask::*,
};

/// an object which writes file permissions (mode, owner, group)
pub struct PermWriter<'s> {
    pub skin: &'s StyleMap,
    max_user_len: usize,
    max_group_len: usize,
}

impl<'s> PermWriter<'s> {

    pub fn new(
        skin: &'s StyleMap,
        max_user_len: usize,
        max_group_len: usize,
    ) -> Self {
        Self { skin, max_user_len, max_group_len }
    }

    pub fn for_tree(
        skin: &'s StyleMap,
        tree: &Tree,
    ) -> Self {
        let (max_user_len, max_group_len) = user_group_max_lengths(tree);
        Self::new(skin, max_user_len, max_group_len)
    }

    fn write_mode<W: Write>(
        &self,
        cw: &mut CropWriter<W>,
        mode: Mode,
        selected: bool,
    ) -> Result<(), termimad::Error> {
        cond_bg!(n_style, self, selected, self.skin.perm__);
        cond_bg!(r_style, self, selected, self.skin.perm_r);
        cond_bg!(w_style, self, selected, self.skin.perm_w);
        cond_bg!(x_style, self, selected, self.skin.perm_x);

        if mode.has(USER_READ) {
            cw.queue_char(r_style, 'r')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }
        if mode.has(USER_WRITE) {
            cw.queue_char(w_style, 'w')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }
        if mode.has(USER_EXEC) {
            cw.queue_char(x_style, 'x')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }

        if mode.has(GROUP_READ) {
            cw.queue_char(r_style, 'r')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }
        if mode.has(GROUP_WRITE) {
            cw.queue_char(w_style, 'w')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }
        if mode.has(GROUP_EXEC) {
            cw.queue_char(x_style, 'x')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }

        if mode.has(OTHERS_READ) {
            cw.queue_char(r_style, 'r')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }
        if mode.has(OTHERS_WRITE) {
            cw.queue_char(w_style, 'w')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }
        if mode.has(OTHERS_EXEC) {
            cw.queue_char(x_style, 'x')?;
        } else {
            cw.queue_char(n_style, '_')?;
        }

        Ok(())
    }

    #[cfg(not(any(target_family = "windows", target_os = "android")))]
    pub fn write_permissions<W: Write>(
        &self,
        cw: &mut CropWriter<W>,
        line: &TreeLine,
        selected: bool,
    ) -> Result<usize, ProgramError> {
        Ok(if line.is_selectable() {
            self.write_mode(cw, line.mode(), selected)?;
            let owner = permissions::user_name(line.metadata.uid());
            cond_bg!(owner_style, self, selected, self.skin.owner);
            cw.queue_g_string(
                owner_style,
                format!(" {:w$}", &owner, w = self.max_user_len),
            )?;
            let group = permissions::group_name(line.metadata.gid());
            cond_bg!(group_style, self, selected, self.skin.group);
            cw.queue_g_string(
                group_style,
                format!(" {:w$}", &group, w = self.max_group_len),
            )?;
            1
        } else {
            9 + 1 + self.max_user_len + 1 + self.max_group_len + 1
        })
    }

}

fn user_group_max_lengths(tree: &Tree) -> (usize, usize) {
    let mut max_user_len = 0;
    let mut max_group_len = 0;
    if tree.options.show_permissions {
        for i in 1..tree.lines.len() {
            let line = &tree.lines[i];
            let user = permissions::user_name(line.metadata.uid());
            max_user_len = max_user_len.max(user.len());
            let group = permissions::group_name(line.metadata.gid());
            max_group_len = max_group_len.max(group.len());
        }
    }
    (max_user_len, max_group_len)
}
