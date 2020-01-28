/// this module generate the clap App, which defines
/// launch arguments

use {
    clap,
};

/// declare the possible CLI arguments, and gets the values
pub fn clap_app() -> clap::App<'static, 'static> {
    clap::App::new("broot")
        .version(env!("CARGO_PKG_VERSION"))
        .author("dystroy <denys.seguret@gmail.com>")
        .about("Balanced tree view + fuzzy search + BFS + customizable launcher")
        .arg(clap::Arg::with_name("root").help("sets the root directory"))
        .arg(
            clap::Arg::with_name("cmd_export_path")
                .long("outcmd")
                .takes_value(true)
                .help("where to write the produced cmd (if any)"),
        )
        .arg(
            clap::Arg::with_name("commands")
                .short("c")
                .long("cmd")
                .takes_value(true)
                .help("semicolon separated commands to execute (experimental)"),
        )
        .arg(
            clap::Arg::with_name("conf")
                .long("conf")
                .takes_value(true)
                .help("semicolon separated paths to specific config files"),
        )
        .arg(
            clap::Arg::with_name("dates")
                .short("d")
                .long("dates")
                .help("show the last modified date of files and directories"),
        )
        .arg(
            clap::Arg::with_name("file_export_path")
                .short("o")
                .long("out")
                .takes_value(true)
                .help("where to write the produced path (if any)"),
        )
        .arg(
            clap::Arg::with_name("show-gitignored")
                .short("i")
                .long("show-gitignored")
                .help("show files which should be ignored according to git"),
        )
        .arg(
            clap::Arg::with_name("height")
                .long("height")
                .help("height (if you don't want to fill the screen or for file export)")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("hidden")
                .short("h")
                .long("hidden")
                .help("show hidden files"),
        )
        .arg(
            clap::Arg::with_name("install")
                .long("install")
                .help("install or reinstall the br shell function"),
        )
        .arg(
            clap::Arg::with_name("no-style")
                .long("no-style")
                .help("whether to remove all style and colors"),
        )
        .arg(
            clap::Arg::with_name("only-folders")
                .short("f")
                .long("only-folders")
                .help("only show folders"),
        )
        .arg(
            clap::Arg::with_name("permissions")
                .short("p")
                .long("permissions")
                .help("show permissions, with owner and group"),
        )
        .arg(
            clap::Arg::with_name("set-install-state")
                .long("set-install-state")
                .takes_value(true)
                .possible_values(&["undefined", "refused", "installed"])
                .help("set the installation state (for use in install script)"),
        )
        .arg(
            clap::Arg::with_name("print-shell-function")
                .long("print-shell-function")
                .takes_value(true)
                .help("print to stdout the br function for a given shell"),
        )
        .arg(
            clap::Arg::with_name("sizes")
                .short("s")
                .long("sizes")
                .help("show the size of files and directories"),
        )
}
