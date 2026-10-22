use std::str::FromStr;

macro_rules! standard_args {
    (
        $($variant:ident: $name:literal $needs_selection:literal,)*
    ) => {
        /// An argument whose value is provided by broot, eg `{file}` or `{root}`
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum StandardArg {
            $($variant,)*
        }
        impl StandardArg {
            /// Tell whether the value depends on the current selection
            pub fn needs_selection(self) -> bool {
                match self {
                    $(Self::$variant => $needs_selection,)*
                }
            }
        }
        impl FromStr for StandardArg {
            type Err = ();
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($name => Ok(Self::$variant),)*
                    _ => Err(()),
                }
            }
        }
    }
}

standard_args! {
    // Variant: "name in patterns" needs_selection
    // (needs_selection: whether the value depends on the current selection)
    Root: "root" false,
    InitialRoot: "initial-root" false,
    Line: "line" true,
    File: "file" true,
    FileName: "file-name" true,
    FileStem: "file-stem" true,
    FileExtension: "file-extension" true,
    FileDotExtension: "file-dot-extension" true,
    Directory: "directory" true,
    Parent: "parent" true,
    OtherPanelFile: "other-panel-file" false,
    OtherPanelFilename: "other-panel-filename" false,
    OtherPanelDirectory: "other-panel-directory" false,
    OtherPanelParent: "other-panel-parent" false,
    GitRoot: "git-root" true,
    GitName: "git-name" true,
    FileGitRelative: "file-git-relative" true,
    FileRootRelative: "file-root-relative" true,
    ServerName: "server-name" false,
}

impl StandardArg {
    /// Tell whether the value comes from the selection in the other panel
    pub fn needs_other_panel(self) -> bool {
        matches!(
            self,
            Self::OtherPanelFile
                | Self::OtherPanelFilename
                | Self::OtherPanelDirectory
                | Self::OtherPanelParent
        )
    }
}
