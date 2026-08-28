use {
    gix::{
        Repository,
        bstr::{
            BStr,
            BString,
        },
        diff::{
            blob::{
                DiffLineStats,
                pipeline::{
                    Mode,
                    WorktreeRoots,
                },
            },
            index::Change,
        },
        dir::{
            entry::Status as DirEntryStatus,
            walk::{
                CollapsedEntriesEmissionMode,
                EmissionMode,
            },
        },
        object::tree::EntryKind,
        status::{
            Item,
            Submodule,
            UntrackedFiles,
            index_worktree,
            index_worktree::iter::Summary,
        },
    },
    rustc_hash::FxHashMap,
    std::{
        collections::hash_map::Entry,
        path::{
            Path,
            PathBuf,
        },
    },
};

/// The git status of a file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineGitStatus {
    /// untracked
    New,
    /// staged, absent from HEAD
    Added,
    /// staged modification, worktree identical to the index
    Staged,
    /// modified in the worktree (staged or not)
    Modified,
    /// staged rename
    Renamed,
    Conflicted,
    Ignored,
    Other,
}

impl LineGitStatus {
    fn from_item(item: &Item) -> Option<Self> {
        match item {
            Item::IndexWorktree(item) => Self::from_index_worktree_item(item),
            Item::TreeIndex(change) => Some(match change {
                Change::Addition { .. } => Self::Added,
                Change::Modification { .. } => Self::Staged,
                Change::Rewrite { .. } => Self::Renamed,
                Change::Deletion { .. } => return None,
            }),
        }
    }
    fn from_index_worktree_item(item: &index_worktree::Item) -> Option<Self> {
        if let index_worktree::Item::DirectoryContents { entry, .. } = item {
            if let DirEntryStatus::Ignored(_) = entry.status {
                return Some(Self::Ignored);
            }
        }
        Some(match item.summary()? {
            Summary::Added | Summary::IntentToAdd => Self::New,
            Summary::Modified => Self::Modified,
            Summary::Conflict => Self::Conflicted,
            Summary::Removed => return None,
            _ => Self::Other,
        })
    }
    /// Return the precedence of the status, used when a path has both a
    /// HEAD->index and an index->worktree change
    fn rank(self) -> u8 {
        match self {
            Self::Conflicted => 7,
            Self::Modified => 6,
            Self::Renamed => 5,
            Self::New => 4,
            Self::Staged | Self::Added => 3,
            Self::Ignored => 2,
            Self::Other => 1,
        }
    }
}

/// As a git repo can't tell whether a path has a status, this computer
/// looks at all the statuses of the repo and build a map path->status
/// which can then be efficiently queried
pub struct LineStatusComputer {
    interesting_statuses: FxHashMap<PathBuf, LineGitStatus>,
}
impl LineStatusComputer {
    /// Build the map of the statuses of the files of the tree rooted at `root`
    pub fn from_root(root: &Path) -> Option<Self> {
        let repo = super::discover(root)?;
        Self::from(&repo, root)
    }
    fn from(
        repo: &Repository,
        root: &Path,
    ) -> Option<Self> {
        let workdir = repo.workdir()?;
        let items = repo
            .status(gix::progress::Discard)
            .ok()?
            // untracked directories are reported as a whole, and their content too
            .untracked_files(UntrackedFiles::Collapsed)
            // we only need to know whether a submodule is dirty
            .index_worktree_submodules(Submodule::AsConfigured { check_dirty: true })
            .dirwalk_options(|o| {
                o.emit_ignored(Some(EmissionMode::Matching))
                    .emit_collapsed(Some(CollapsedEntriesEmissionMode::All))
            })
            .into_iter(pathspecs(workdir, root))
            .ok()?;
        let mut interesting_statuses: FxHashMap<PathBuf, LineGitStatus> = FxHashMap::default();
        for item in items {
            let item = match item {
                Ok(item) => item,
                Err(e) => {
                    debug!("git status item error: {e:?}");
                    continue;
                }
            };
            let Some(status) = LineGitStatus::from_item(&item) else {
                continue;
            };
            let path = workdir.join(gix::path::from_bstr(item.location()));
            match interesting_statuses.entry(path) {
                Entry::Vacant(e) => {
                    e.insert(status);
                }
                Entry::Occupied(mut e) => {
                    if status.rank() > e.get().rank() {
                        e.insert(status);
                    }
                }
            }
        }
        Some(Self {
            interesting_statuses,
        })
    }
    pub fn line_status(
        &self,
        path: &Path,
    ) -> Option<LineGitStatus> {
        self.interesting_statuses.get(path).copied()
    }
    pub fn is_interesting(
        &self,
        path: &Path,
    ) -> bool {
        self.interesting_statuses.contains_key(path)
    }
}

/// Return the pathspec limiting the status to the tree root, when
/// it's not the repository root
fn pathspecs(
    workdir: &Path,
    root: &Path,
) -> Vec<BString> {
    root.strip_prefix(workdir)
        .ok()
        .filter(|rel| !rel.as_os_str().is_empty())
        .map(|rel| gix::path::to_unix_separators_on_windows(gix::path::into_bstr(rel)).into_owned())
        .into_iter()
        .collect()
}

#[derive(Debug, Clone)]
pub struct TreeGitStatus {
    pub current_branch_name: Option<String>,
    pub insertions: usize,
    pub deletions: usize,
    /// commits of HEAD not in the upstream branch (0 when there's no upstream)
    pub ahead: usize,
    /// commits of the upstream branch not in HEAD (0 when there's no upstream)
    pub behind: usize,
}

impl TreeGitStatus {
    pub fn from(repo: &Repository) -> Option<Self> {
        let current_branch_name = match repo.head_name() {
            Ok(Some(name)) => Some(name.shorten().to_string()),
            Ok(None) => repo.head_id().ok().map(|id| id.shorten_or_id().to_string()),
            Err(e) => {
                debug!("get head failed : {e:?}");
                None
            }
        };
        let (insertions, deletions) = match index_to_workdir_stats(repo) {
            Ok(stats) => stats,
            Err(e) => {
                debug!("get diff stats failed : {e:?}");
                return None;
            }
        };
        let (ahead, behind) = ahead_behind(repo).unwrap_or((0, 0));
        Some(Self {
            current_branch_name,
            insertions,
            deletions,
            ahead,
            behind,
        })
    }
}

/// Count the commits of HEAD which aren't in the upstream branch, and
/// the commits of the upstream branch which aren't in HEAD.
/// None when there's no upstream branch.
fn ahead_behind(repo: &Repository) -> Option<(usize, usize)> {
    let head_ref = repo.head_ref().ok()??;
    let upstream_name = head_ref
        .remote_tracking_ref_name(gix::remote::Direction::Fetch)?
        .ok()?;
    let upstream_id = repo
        .find_reference(upstream_name.as_ref())
        .ok()?
        .into_fully_peeled_id()
        .ok()?
        .detach();
    let head_id = repo.head_id().ok()?.detach();
    if head_id == upstream_id {
        return Some((0, 0));
    }
    let count = |from, hidden| -> Option<usize> {
        let walk = repo.rev_walk([from]).with_hidden([hidden]).all().ok()?;
        Some(walk.filter(Result::is_ok).count())
    };
    Some((count(head_id, upstream_id)?, count(upstream_id, head_id)?))
}

/// Count the lines inserted and deleted in the worktree compared to the index
fn index_to_workdir_stats(repo: &Repository) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    use gix::status::index_worktree::iter::Summary::*;
    let Some(workdir) = repo.workdir() else {
        return Ok((0, 0));
    };
    let null = repo.object_hash().null();
    // the resources of this cache are read from the worktree
    let mut wt_cache = repo.diff_resource_cache(
        Mode::ToGit,
        WorktreeRoots {
            old_root: None,
            new_root: Some(workdir.to_path_buf()),
        },
    )?;
    // the resources of this cache are read from the object database
    let mut odb_cache = repo.diff_resource_cache(Mode::ToGit, WorktreeRoots::default())?;
    let items = repo
        .status(gix::progress::Discard)?
        .untracked_files(UntrackedFiles::None)
        .into_index_worktree_iter(Vec::<BString>::new())?;
    let mut insertions = 0;
    let mut deletions = 0;
    for item in items {
        let item = item?;
        let index_worktree::Item::Modification {
            entry, rela_path, ..
        } = &item
        else {
            continue;
        };
        let Some(kind) = entry.mode.to_tree_entry_mode().map(|m| m.kind()) else {
            continue;
        };
        if !matches!(
            kind,
            EntryKind::Blob | EntryKind::BlobExecutable | EntryKind::Link
        ) {
            continue; // submodules
        }
        let stats = match item.summary() {
            Some(Modified) => line_counts(
                &mut wt_cache,
                entry.id,
                null,
                kind,
                rela_path.as_ref(),
                repo,
            ),
            Some(Removed) => line_counts(
                &mut odb_cache,
                entry.id,
                null,
                kind,
                rela_path.as_ref(),
                repo,
            ),
            Some(IntentToAdd) => {
                line_counts(&mut wt_cache, null, null, kind, rela_path.as_ref(), repo)
            }
            _ => continue,
        };
        match stats {
            Ok(Some(stats)) => {
                insertions += stats.insertions as usize;
                deletions += stats.removals as usize;
            }
            Ok(None) => {} // binary
            Err(e) => {
                debug!("diff failed for {rela_path}: {e:?}");
            }
        }
    }
    Ok((insertions, deletions))
}

/// Diff the two versions of a file, identified by their object id, a null id
/// meaning either the worktree file if the cache has a worktree root for
/// this side, or a missing file otherwise.
fn line_counts(
    cache: &mut gix::diff::blob::Platform,
    old_id: gix::ObjectId,
    new_id: gix::ObjectId,
    kind: EntryKind,
    rela_path: &BStr,
    repo: &Repository,
) -> Result<Option<DiffLineStats>, Box<dyn std::error::Error>> {
    use gix::diff::blob::ResourceKind::*;
    cache.set_resource(old_id, kind, rela_path, OldOrSource, repo)?;
    cache.set_resource(new_id, kind, rela_path, NewOrDestination, repo)?;
    let mut platform = gix::object::blob::diff::Platform {
        resource_cache: cache,
    };
    Ok(platform.line_counts()?)
}
