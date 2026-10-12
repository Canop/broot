//! Tests of the expansion of execution patterns, covering the standard arguments,
//! the invocation arguments, the flags, the selection kinds, and the targets
//! (direct execution or shell command line).

use {
    super::*,
    crate::{
        app::*,
        command::Sequence,
        conf::Conf,
        stage::Stage,
    },
    std::{
        fs,
        path::{
            Path,
            PathBuf,
        },
    },
    tempfile::TempDir,
};

/// A temporary file tree, with names needing escaping in a shell
struct Tree {
    _tmp: TempDir,
    root: PathBuf,
    /// `<root>/my dir`
    dir: PathBuf,
    /// `<root>/my dir/a b.txt`
    file: PathBuf,
    /// `<root>/my dir/sub dir`
    sub: PathBuf,
    /// `<root>/my dir/it's $x.txt`
    odd: PathBuf,
    /// `<root>/other/o p.md`
    other: PathBuf,
}

impl Tree {
    fn new() -> Self {
        let tmp = tempfile::Builder::new().prefix("broot").tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let dir = root.join("my dir");
        let file = dir.join("a b.txt");
        let sub = dir.join("sub dir");
        let odd = dir.join("it's $x.txt");
        let other = root.join("other").join("o p.md");
        fs::create_dir_all(&sub).unwrap();
        fs::create_dir_all(other.parent().unwrap()).unwrap();
        for f in [&file, &odd, &other] {
            fs::write(f, "").unwrap();
        }
        Self {
            _tmp: tmp,
            root,
            dir,
            file,
            sub,
            odd,
            other,
        }
    }
    fn app_state(&self) -> AppState {
        let mut app_state = AppState::new(&self.root);
        app_state.other_panel_path = Some(self.other.clone());
        app_state
    }
    /// The path relative to the root, as a string
    fn rel(
        &self,
        path: &Path,
    ) -> String {
        path.strip_prefix(&self.root).unwrap().to_string_lossy().to_string()
    }
}

fn s(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn context(tree: &Tree) -> AppContext {
    AppContext {
        initial_root: tree.dir.clone(),
        server_name: Some("srv x".to_string()),
        ..Default::default()
    }
}

fn file_sel(path: &Path) -> Selection<'_> {
    let mut sel = Selection::new(path, SelectionType::File);
    sel.line = 7;
    sel
}

/// Expand the pattern for a direct execution, with a fresh builder
fn tokens(
    sel_info: SelInfo<'_>,
    app_state: &AppState,
    con: &AppContext,
    pattern: &ExecPattern,
) -> Vec<String> {
    ExecutionBuilder::without_invocation(sel_info, app_state).exec_token(pattern, con)
}

/// Expand the pattern for a shell, with a fresh builder
fn shell(
    sel_info: SelInfo<'_>,
    app_state: &AppState,
    con: &AppContext,
    pattern: &ExecPattern,
) -> String {
    ExecutionBuilder::without_invocation(sel_info, app_state).shell_exec_string(pattern, con)
}

fn string_pattern(s: &str) -> ExecPattern {
    ExecPattern::from_string(s)
}

fn array_pattern(tokens: &[&str]) -> ExecPattern {
    ExecPattern::from_tokens(tokens.iter().map(|t| t.to_string()).collect())
}

/// Check the expansion of a single standard argument group, as a direct
/// execution token and in a shell command line, from both pattern forms
fn check_standard_arg(
    sel_info: SelInfo<'_>,
    app_state: &AppState,
    con: &AppContext,
    name: &str,
    token: &str,
    shell_line: &str,
    shell_word: &str,
) {
    let group = format!("{{{name}}}");
    let from_string = string_pattern(&group);
    let from_array = array_pattern(&[&group]);
    assert_eq!(tokens(sel_info, app_state, con, &from_string), vec![token], "token of {name}");
    assert_eq!(tokens(sel_info, app_state, con, &from_array), vec![token], "token of {name}");
    assert_eq!(shell(sel_info, app_state, con, &from_string), shell_line, "shell line of {name}");
    assert_eq!(shell(sel_info, app_state, con, &from_array), shell_word, "shell word of {name}");
}

#[test]
fn standard_args_on_a_file() {
    let tree = Tree::new();
    let app_state = tree.app_state();
    let con = context(&tree);
    let sel_info = SelInfo::One(file_sel(&tree.file));
    let r = s(&tree.root);
    #[rustfmt::skip]
    let cases: Vec<(&str, String, String, String)> = vec![
        // name, token, in a shell line, as a shell word
        ("root", r.clone(), r.clone(), r.clone()),
        ("initial-root", format!("{r}/my dir"), format!("'{r}/my dir'"), format!("'{r}/my dir'")),
        ("line", "7".into(), "7".into(), "7".into()),
        ("file", format!("{r}/my dir/a b.txt"), format!("'{r}/my dir/a b.txt'"), format!("'{r}/my dir/a b.txt'")),
        ("file-name", "a b.txt".into(), "'a b.txt'".into(), "'a b.txt'".into()),
        ("file-stem", "a b".into(), "'a b'".into(), "'a b'".into()),
        ("file-extension", "txt".into(), "txt".into(), "txt".into()),
        ("file-dot-extension", ".txt".into(), ".txt".into(), ".txt".into()),
        ("directory", format!("{r}/my dir"), format!("'{r}/my dir'"), format!("'{r}/my dir'")),
        ("parent", format!("{r}/my dir"), format!("'{r}/my dir'"), format!("'{r}/my dir'")),
        ("other-panel-file", format!("{r}/other/o p.md"), format!("'{r}/other/o p.md'"), format!("'{r}/other/o p.md'")),
        ("other-panel-filename", "o p.md".into(), "'o p.md'".into(), "'o p.md'".into()),
        ("other-panel-directory", format!("{r}/other"), format!("{r}/other"), format!("{r}/other")),
        ("other-panel-parent", format!("{r}/other"), format!("{r}/other"), format!("{r}/other")),
        ("git-root", "".into(), "''".into(), "''".into()),
        ("git-name", "".into(), "''".into(), "''".into()),
        ("file-git-relative", format!("{r}/my dir/a b.txt"), format!("'{r}/my dir/a b.txt'"), format!("'{r}/my dir/a b.txt'")),
        ("file-root-relative", "my dir/a b.txt".into(), "'my dir/a b.txt'".into(), "'my dir/a b.txt'".into()),
        ("server-name", "srv x".into(), "'srv x'".into(), "'srv x'".into()),
    ];
    for (name, token, shell_line, shell_word) in &cases {
        check_standard_arg(sel_info, &app_state, &con, name, token, shell_line, shell_word);
    }
}

#[test]
fn standard_args_on_a_directory() {
    let tree = Tree::new();
    let app_state = tree.app_state();
    let con = context(&tree);
    let sel_info = SelInfo::One(Selection::new(&tree.sub, SelectionType::Directory));
    let r = s(&tree.root);
    #[rustfmt::skip]
    let cases: Vec<(&str, String, String, String)> = vec![
        ("file", format!("{r}/my dir/sub dir"), format!("'{r}/my dir/sub dir'"), format!("'{r}/my dir/sub dir'")),
        ("file-name", "sub dir".into(), "'sub dir'".into(), "'sub dir'".into()),
        ("file-extension", "".into(), "''".into(), "''".into()),
        ("file-dot-extension", "".into(), "''".into(), "''".into()),
        ("directory", format!("{r}/my dir/sub dir"), format!("'{r}/my dir/sub dir'"), format!("'{r}/my dir/sub dir'")),
        ("parent", format!("{r}/my dir"), format!("'{r}/my dir'"), format!("'{r}/my dir'")),
        ("line", "0".into(), "0".into(), "0".into()),
    ];
    for (name, token, shell_line, shell_word) in &cases {
        check_standard_arg(sel_info, &app_state, &con, name, token, shell_line, shell_word);
    }
}

#[test]
fn standard_args_without_selection() {
    let tree = Tree::new();
    let app_state = tree.app_state();
    let con = context(&tree);
    let r = s(&tree.root);
    #[rustfmt::skip]
    let cases: Vec<(&str, String, String, String)> = vec![
        ("root", r.clone(), r.clone(), r.clone()),
        ("file", "".into(), "''".into(), "''".into()),
        ("file-dot-extension", "".into(), "''".into(), "''".into()),
        ("line", "".into(), "''".into(), "''".into()),
        ("other-panel-filename", "o p.md".into(), "'o p.md'".into(), "'o p.md'".into()),
    ];
    for (name, token, shell_line, shell_word) in &cases {
        check_standard_arg(SelInfo::None, &app_state, &con, name, token, shell_line, shell_word);
    }
}

#[test]
fn standard_args_with_special_chars() {
    let tree = Tree::new();
    let app_state = tree.app_state();
    let con = context(&tree);
    let sel_info = SelInfo::One(file_sel(&tree.odd));
    let r = s(&tree.root);
    check_standard_arg(
        sel_info,
        &app_state,
        &con,
        "file",
        &format!("{r}/my dir/it's $x.txt"),
        &format!(r#"'{r}/my dir/it'"'"'s $x.txt'"#),
        &format!(r#"'{r}/my dir/it'"'"'s $x.txt'"#),
    );
    check_standard_arg(
        sel_info,
        &app_state,
        &con,
        "file-name",
        "it's $x.txt",
        r#"'it'"'"'s $x.txt'"#,
        r#"'it'"'"'s $x.txt'"#,
    );
}

#[test]
fn git_args() {
    let tree = Tree::new();
    let repo = tree.root.join("repo");
    let repo_file = repo.join("r s.rs");
    fs::create_dir_all(&repo).unwrap();
    fs::write(&repo_file, "").unwrap();
    gix::init(&repo).unwrap();
    let app_state = AppState::new(&repo);
    let con = context(&tree);
    let sel_info = SelInfo::One(file_sel(&repo_file));
    let r = s(&repo);
    check_standard_arg(sel_info, &app_state, &con, "git-root", &r, &r, &r);
    check_standard_arg(sel_info, &app_state, &con, "git-name", "repo", "repo", "repo");
    check_standard_arg(
        sel_info,
        &app_state,
        &con,
        "file-git-relative",
        "r s.rs",
        "'r s.rs'",
        "'r s.rs'",
    );
}

/// Expand with a builder knowing the invocation pattern and the typed arguments
fn invocation_tokens_and_shell(
    sel_info: SelInfo<'_>,
    app_state: &AppState,
    con: &AppContext,
    invocation: &str,
    args: Option<&str>,
    pattern: &ExecPattern,
) -> (Vec<String>, String) {
    let parser = InvocationParser::new(invocation).unwrap();
    let args = args.map(String::from);
    let tokens = ExecutionBuilder::with_invocation(Some(&parser), sel_info, app_state, args.as_ref())
        .exec_token(pattern, con);
    let shell = ExecutionBuilder::with_invocation(Some(&parser), sel_info, app_state, args.as_ref())
        .shell_exec_string(pattern, con);
    (tokens, shell)
}

#[test]
fn invocation_args() {
    let tree = Tree::new();
    let app_state = tree.app_state();
    let con = context(&tree);
    let sel_info = SelInfo::One(file_sel(&tree.file));
    let r = s(&tree.root);
    let check = |invocation: &str, args: Option<&str>, pattern: &str, token: &str, shell_line: &str| {
        let (t, sh) = invocation_tokens_and_shell(
            sel_info,
            &app_state,
            &con,
            invocation,
            args,
            &string_pattern(pattern),
        );
        assert_eq!(t, vec![token], "token of {pattern} with {args:?}");
        assert_eq!(sh, shell_line, "shell line of {pattern} with {args:?}");
    };
    // value inserted as typed
    check("x {v}", Some("hello  world"), "{v}", "hello  world", "'hello  world'");
    // no typed value
    check("x {v}", None, "{v}", "", "''");
    // typed value not matching the invocation pattern
    check("x {a} {b}", Some("single"), "{a}", "", "''");
    // unknown group
    check("x {v}", Some("val"), "{nope}", "", "''");
    // an invocation argument masks the standard argument with the same name
    check("x {file}", Some("typed"), "{file}", "typed", "typed");
    check("x {file}", None, "{file}", "", "''");
    // path flags, from a file
    let home = s(&crate::path::untilde("~"));
    for flag in ["path-from-directory", "path-from-parent"] {
        let pattern = format!("{{p:{flag}}}");
        check("x {p}", Some("new dir/f"), &pattern, &format!("{r}/my dir/new dir/f"), &format!("'{r}/my dir/new dir/f'"));
        check("x {p}", Some("../up"), &pattern, &format!("{r}/up"), &format!("{r}/up"));
        check("x {p}", Some("/abs x"), &pattern, "/abs x", "'/abs x'");
        check("x {p}", Some("~/h"), &pattern, &format!("{home}/h"), &format!("{home}/h"));
    }
}

#[test]
fn path_flags_from_a_directory() {
    let tree = Tree::new();
    let app_state = tree.app_state();
    let con = context(&tree);
    let r = s(&tree.root);
    let check = |sel_info: SelInfo<'_>, pattern: &str, token: &str| {
        let (t, _) = invocation_tokens_and_shell(
            sel_info,
            &app_state,
            &con,
            "x {p}",
            Some("n"),
            &string_pattern(pattern),
        );
        assert_eq!(t, vec![token], "token of {pattern}");
    };
    let sub = SelInfo::One(Selection::new(&tree.sub, SelectionType::Directory));
    check(sub, "{p:path-from-directory}", &format!("{r}/my dir/sub dir/n"));
    check(sub, "{p:path-from-parent}", &format!("{r}/my dir/n"));
    let fs_root = Path::new("/");
    let top = SelInfo::One(Selection::new(fs_root, SelectionType::Directory));
    check(top, "{p:path-from-directory}", "/n");
    check(top, "{p:path-from-parent}", "");
}

#[test]
fn quotes_and_shell_syntax() {
    let tree = Tree::new();
    let app_state = tree.app_state();
    let con = context(&tree);
    let sel_info = SelInfo::One(file_sel(&tree.file));
    let r = s(&tree.root);
    let p = string_pattern(r#"touch "c d" {file}"#);
    assert_eq!(tokens(sel_info, &app_state, &con, &p), vec!["touch", "c d", &format!("{r}/my dir/a b.txt")]);
    assert_eq!(shell(sel_info, &app_state, &con, &p), format!(r#"touch "c d" '{r}/my dir/a b.txt'"#));
    let p = string_pattern(r#"echo "$HOME"  'x  y' {file} | wc && ls"#);
    assert_eq!(
        shell(sel_info, &app_state, &con, &p),
        format!(r#"echo "$HOME"  'x  y' '{r}/my dir/a b.txt' | wc && ls"#),
    );
    let p = array_pattern(&["touch", "c d", "{file}"]);
    assert_eq!(tokens(sel_info, &app_state, &con, &p), vec!["touch", "c d", &format!("{r}/my dir/a b.txt")]);
    assert_eq!(shell(sel_info, &app_state, &con, &p), format!("touch 'c d' '{r}/my dir/a b.txt'"));
    // a pattern read from the configuration as a string
    let p: ExecPattern = deser_hjson::from_str(r#" "touch \"c d\" {file}" "#).unwrap();
    assert_eq!(tokens(sel_info, &app_state, &con, &p), vec!["touch", "c d", &format!("{r}/my dir/a b.txt")]);
    assert_eq!(shell(sel_info, &app_state, &con, &p), format!(r#"touch "c d" '{r}/my dir/a b.txt'"#));
    // a group inside a token
    let p = string_pattern("cp {file} {directory}/copy.txt --name={file-name}");
    assert_eq!(
        tokens(sel_info, &app_state, &con, &p),
        vec![
            "cp".to_string(),
            format!("{r}/my dir/a b.txt"),
            format!("{r}/my dir/copy.txt"),
            "--name=a b.txt".to_string(),
        ],
    );
    assert_eq!(
        shell(sel_info, &app_state, &con, &p),
        format!("cp '{r}/my dir/a b.txt' '{r}/my dir'/copy.txt --name='a b.txt'"),
    );
}

#[test]
fn tilde_tokens() {
    let tree = Tree::new();
    let app_state = tree.app_state();
    let con = context(&tree);
    let sel_info = SelInfo::One(file_sel(&tree.file));
    let home = s(&crate::path::untilde("~"));
    let p = string_pattern("ls ~ ~/nonexistent-broot-x");
    assert_eq!(tokens(sel_info, &app_state, &con, &p), vec!["ls", &home, "~/nonexistent-broot-x"]);
    assert_eq!(shell(sel_info, &app_state, &con, &p), "ls ~ ~/nonexistent-broot-x");
    let p = array_pattern(&["ls", "~"]);
    assert_eq!(shell(sel_info, &app_state, &con, &p), format!("ls {home}"));
}

fn staged(tree: &Tree) -> Stage {
    let mut stage = Stage::default();
    stage.add(tree.file.clone());
    stage.add(tree.odd.clone());
    stage
}

#[test]
fn merging_flags() {
    let tree = Tree::new();
    let mut app_state = tree.app_state();
    app_state.stage = staged(&tree);
    let con = context(&tree);
    let sel_info = SelInfo::More(&app_state.stage);
    let r = s(&tree.root);
    let file = format!("{r}/my dir/a b.txt");
    let odd = format!("{r}/my dir/it's $x.txt");
    let file_q = format!("'{r}/my dir/a b.txt'");
    let odd_q = format!(r#"'{r}/my dir/it'"'"'s $x.txt'"#);

    let p = string_pattern("mpv {file:space-separated}");
    assert_eq!(tokens(sel_info, &app_state, &con, &p), vec!["mpv", &file, &odd]);
    assert_eq!(shell(sel_info, &app_state, &con, &p), format!("mpv {file_q} {odd_q}"));
    let p = array_pattern(&["mpv", "{file:space-separated}"]);
    assert_eq!(shell(sel_info, &app_state, &con, &p), format!("mpv {file_q} {odd_q}"));

    let p = string_pattern("mpv --f={file:space-separated}");
    assert_eq!(tokens(sel_info, &app_state, &con, &p), vec!["mpv".to_string(), format!("--f={file} {odd}")]);
    assert_eq!(shell(sel_info, &app_state, &con, &p), format!("mpv --f={file_q} {odd_q}"));

    let p = string_pattern("smerge {file:comma-separated}");
    assert_eq!(tokens(sel_info, &app_state, &con, &p), vec!["smerge".to_string(), format!("{file},{odd}")]);
    assert_eq!(shell(sel_info, &app_state, &con, &p), format!("smerge {file_q},{odd_q}"));
    let p = array_pattern(&["smerge", "{file:comma-separated}"]);
    assert_eq!(
        shell(sel_info, &app_state, &con, &p),
        format!(r#"smerge '{r}/my dir/a b.txt,{r}/my dir/it'"'"'s $x.txt'"#),
    );

    // a group without merging flag takes the value of the first selection
    let p = string_pattern("zip {file:space-separated} {parent}");
    assert_eq!(tokens(sel_info, &app_state, &con, &p), vec!["zip", &file, &odd, &format!("{r}/my dir")]);
}

#[test]
fn per_selection_expansion() {
    let tree = Tree::new();
    let mut app_state = tree.app_state();
    app_state.stage = staged(&tree);
    let con = context(&tree);
    let r = s(&tree.root);
    let sel = Selection::from_path(&tree.odd);
    for p in [string_pattern("cp {file} {parent}"), array_pattern(&["cp", "{file}", "{parent}"])] {
        let builder = ExecutionBuilder::without_invocation(SelInfo::More(&app_state.stage), &app_state);
        assert_eq!(
            builder.sel_exec_token(&p, Some(sel), &con),
            vec!["cp".to_string(), format!("{r}/my dir/it's $x.txt"), format!("{r}/my dir")],
        );
        let builder = ExecutionBuilder::without_invocation(SelInfo::More(&app_state.stage), &app_state);
        assert_eq!(
            builder.sel_shell_exec_string(&p, Some(sel), &con),
            format!(r#"cp '{r}/my dir/it'"'"'s $x.txt' '{r}/my dir'"#),
        );
    }
}

#[test]
fn shell_expansion_then_direct_expansion_with_the_same_builder() {
    let tree = Tree::new();
    let app_state = tree.app_state();
    let con = context(&tree);
    let r = s(&tree.root);
    let builder = ExecutionBuilder::without_invocation(SelInfo::One(file_sel(&tree.file)), &app_state);
    builder.shell_exec_string(&string_pattern("echo {file}"), &con);
    assert_eq!(
        builder.exec_token(&string_pattern("{file}"), &con),
        vec![format!("{r}/my dir/a b.txt")],
    );
}

#[test]
fn working_dir_path() {
    let tree = Tree::new();
    let app_state = tree.app_state();
    let con = context(&tree);
    let builder = ExecutionBuilder::without_invocation(SelInfo::One(file_sel(&tree.file)), &app_state);
    assert_eq!(builder.path("{directory}", &con), tree.dir);
    assert_eq!(builder.path("{root}", &con), tree.root);
    assert_eq!(builder.path("sub dir", &con), tree.sub);
    assert_eq!(builder.path("{file-stem}", &con), tree.dir.join("a b"));
    let builder = ExecutionBuilder::without_invocation(SelInfo::None, &app_state);
    assert_eq!(builder.path("sub", &con), tree.root.join("sub"));
}

#[test]
fn sequences() {
    let tree = Tree::new();
    let app_state = tree.app_state();
    let con = context(&tree);
    let r = s(&tree.root);
    let mut conf = Conf::default();
    let verb_store = VerbStore::new(&mut conf).unwrap();
    let sequence = Sequence::new(":focus {directory};:cp {file}  {root}/x", Some(";"));
    let builder = ExecutionBuilder::without_invocation(SelInfo::One(file_sel(&tree.file)), &app_state);
    let sequence = builder.sequence(&sequence, &verb_store, &con, None);
    assert_eq!(sequence.raw, format!(":focus {r}/my dir;:cp '{r}/my dir/a b.txt'  {r}/x"));
    assert_eq!(sequence.separator, ";");
}

#[test]
fn invocation_defaults() {
    let tree = Tree::new();
    let app_state = tree.app_state();
    let con = context(&tree);
    let r = s(&tree.root);
    let builder = ExecutionBuilder::without_invocation(SelInfo::One(file_sel(&tree.file)), &app_state);
    let check = |invocation: &str, args: &str| {
        let parser = InvocationParser::new(invocation).unwrap();
        let filled = builder.invocation_with_default(&parser, &con);
        assert_eq!(filled.args.as_deref(), Some(args), "defaults of {invocation}");
    };
    check("rename {new_filename:file-name}", "a b.txt");
    check("set_syntax_theme {theme:theme}", "");
    check("x {a} {b:parent}", &format!(" {r}/my dir"));
    check("x {a:file-root-relative}", &tree.rel(&tree.file));
}
