use {
    crate::{
        errors::ProgramError,
        syntactic::is_char_unprintable,
    },
    gix::{
        diff::blob::{
            Diff,
            InternedInput,
            ResourceKind,
            pipeline::{
                Mode,
                WorktreeRoots,
            },
            platform::prepare_diff::Operation,
        },
        object::tree::EntryKind,
    },
    std::{
        fmt::Display,
        ops::Range,
        path::{
            Path,
            PathBuf,
        },
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    Context,
    Added,
    Removed,
}

/// A line of a diff hunk
#[derive(Debug, Clone)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    /// the number of the line in the old version (none for an added line)
    pub old_number: Option<usize>,
    /// the number of the line in the new version (none for a removed line)
    pub new_number: Option<usize>,
    pub content: String,
}

/// A group of changes, with the unchanged lines around them
#[derive(Debug, Clone)]
pub struct Hunk {
    pub old_start: usize,
    pub old_len: usize,
    pub new_start: usize,
    pub new_len: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone)]
pub enum FileDiffContent {
    Text(Vec<Hunk>),
    Binary,
    Unchanged,
}

/// The changes of one file between two versions
#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: PathBuf,
    pub content: FileDiffContent,
    pub insertions: usize,
    pub deletions: usize,
}

fn git_error<E: Display>(e: E) -> ProgramError {
    ProgramError::Git {
        details: e.to_string(),
    }
}

/// Compute the changes of the file in the worktree, compared to HEAD.
/// `context` is the number of unchanged lines kept around the changes.
pub fn diff_worktree_vs_head(
    path: &Path,
    context: usize,
) -> Result<FileDiff, ProgramError> {
    let repo = super::discover(path).ok_or_else(|| git_error("not in a git repository"))?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| git_error("bare repository"))?;
    let rela_path = path.strip_prefix(workdir).map_err(git_error)?;
    let rela_bstr = gix::path::to_unix_separators_on_windows(gix::path::into_bstr(rela_path));
    let null = repo.object_hash().null();
    let (old_id, old_kind) = match repo
        .head_tree()
        .map_err(git_error)?
        .lookup_entry_by_path(rela_path)
        .map_err(git_error)?
    {
        Some(entry) => (entry.object_id(), entry.mode().kind()),
        None => (null, EntryKind::Blob), // not in HEAD: everything is added
    };
    let mut cache = repo
        .diff_resource_cache(
            Mode::ToWorktreeAndBinaryToText,
            WorktreeRoots {
                old_root: None,
                new_root: Some(workdir.to_path_buf()),
            },
        )
        .map_err(git_error)?;
    cache
        .set_resource(
            old_id,
            old_kind,
            rela_bstr.as_ref(),
            ResourceKind::OldOrSource,
            &repo,
        )
        .map_err(git_error)?;
    cache
        .set_resource(
            null,
            EntryKind::Blob,
            rela_bstr.as_ref(),
            ResourceKind::NewOrDestination,
            &repo,
        )
        .map_err(git_error)?;
    cache.options.skip_internal_diff_if_external_is_configured = false;
    let prep = cache.prepare_diff().map_err(git_error)?;
    let content = match prep.operation {
        Operation::InternalDiff { algorithm } => {
            let input = prep.interned_input();
            let diff = gix::diff::blob::diff_with_slider_heuristics(algorithm, &input);
            let hunks = hunks(&diff, &input, context);
            if hunks.is_empty() {
                FileDiffContent::Unchanged
            } else {
                FileDiffContent::Text(hunks)
            }
        }
        Operation::SourceOrDestinationIsBinary => FileDiffContent::Binary,
        Operation::ExternalCommand { .. } => {
            return Err(git_error("unexpected external diff"));
        }
    };
    let (insertions, deletions) = match &content {
        FileDiffContent::Text(hunks) => hunks
            .iter()
            .flat_map(|hunk| &hunk.lines)
            .fold((0, 0), |(ins, del), line| match line.kind {
                DiffLineKind::Added => (ins + 1, del),
                DiffLineKind::Removed => (ins, del + 1),
                DiffLineKind::Context => (ins, del),
            }),
        _ => (0, 0),
    };
    Ok(FileDiff {
        path: path.to_path_buf(),
        content,
        insertions,
        deletions,
    })
}

/// Build the hunks of a diff, each change surrounded by `context` unchanged
/// lines, changes closer than that being grouped in the same hunk
fn hunks(
    diff: &Diff,
    input: &InternedInput<&[u8]>,
    context: usize,
) -> Vec<Hunk> {
    let text = |line: &[u8]| -> String {
        let s = String::from_utf8_lossy(line);
        if s.contains(is_char_unprintable) {
            s.replace(is_char_unprintable, "�")
        } else {
            s.into_owned()
        }
    };
    let before_text = |idx: usize| text(input.interner[input.before[idx]]);
    let after_text = |idx: usize| text(input.interner[input.after[idx]]);
    let mut hunks = Vec::new();
    let mut hunk = HunkBuilder::default();
    // indexes of the first lines after the previous change
    let mut done_before = 0;
    let mut done_after = 0;
    for change in diff.hunks() {
        let b: Range<usize> = change.before.start as usize..change.before.end as usize;
        let a: Range<usize> = change.after.start as usize..change.after.end as usize;
        let gap = b.start - done_before; // unchanged lines since the previous change
        if hunk.is_empty() || gap > 2 * context {
            if !hunk.is_empty() {
                for i in 0..context.min(gap) {
                    hunk.push_context(done_before + i, done_after + i, before_text(done_before + i));
                }
                hunks.push(hunk.build());
            }
            let ctx = context.min(gap);
            for i in (b.start - ctx)..b.start {
                hunk.push_context(i, a.start - (b.start - i), before_text(i));
            }
        } else {
            // the changes are close: all the lines between them are context
            for i in 0..gap {
                hunk.push_context(done_before + i, done_after + i, before_text(done_before + i));
            }
        }
        for i in b.clone() {
            hunk.push_removed(i, before_text(i));
        }
        for i in a.clone() {
            hunk.push_added(i, after_text(i));
        }
        done_before = b.end;
        done_after = a.end;
    }
    if !hunk.is_empty() {
        let gap = input.before.len() - done_before;
        for i in 0..context.min(gap) {
            hunk.push_context(done_before + i, done_after + i, before_text(done_before + i));
        }
        hunks.push(hunk.build());
    }
    hunks
}

#[derive(Default)]
struct HunkBuilder {
    lines: Vec<DiffLine>,
}

impl HunkBuilder {
    fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
    /// Push an unchanged line, given its 0-based indexes in both versions
    fn push_context(
        &mut self,
        old_idx: usize,
        new_idx: usize,
        content: String,
    ) {
        self.lines.push(DiffLine {
            kind: DiffLineKind::Context,
            old_number: Some(old_idx + 1),
            new_number: Some(new_idx + 1),
            content,
        });
    }
    fn push_removed(
        &mut self,
        old_idx: usize,
        content: String,
    ) {
        self.lines.push(DiffLine {
            kind: DiffLineKind::Removed,
            old_number: Some(old_idx + 1),
            new_number: None,
            content,
        });
    }
    fn push_added(
        &mut self,
        new_idx: usize,
        content: String,
    ) {
        self.lines.push(DiffLine {
            kind: DiffLineKind::Added,
            old_number: None,
            new_number: Some(new_idx + 1),
            content,
        });
    }
    /// Return the hunk made of the pushed lines, leaving the builder empty
    fn build(&mut self) -> Hunk {
        let lines = std::mem::take(&mut self.lines);
        let old_numbers = lines.iter().filter_map(|l| l.old_number);
        let new_numbers = lines.iter().filter_map(|l| l.new_number);
        Hunk {
            old_start: old_numbers.clone().next().unwrap_or(0),
            old_len: old_numbers.count(),
            new_start: new_numbers.clone().next().unwrap_or(0),
            new_len: new_numbers.count(),
            lines,
        }
    }
}
