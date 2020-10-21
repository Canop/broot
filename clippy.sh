# run clippy, with some lints disabled:
#
# clippy_regex_macro
#    the regex_macro clippy lint doesn't check *what* macro
#    it finds so it produces a lot of false positives.
#
# clippy::collapsible_if
#    my experience is that the collapsible_if lint is mostly
#    a nuisance which pushes towards hiding intent in code.
#
# clippy::module_inception
#    I don't know whether it's bad or not to have sub modules
#    named the same as their parent, and I'm willing to discuss
#    it if you want, but I don't need clippy to tell me every
#    time.
#

# there's a issue somewhere about clippy unable to run
# its lints twice. This part will disappear when the issue
# is solved
rm -rf target
cargo build


# do the clippy
cargo clippy -- -A clippy::match_like_matches_macro -A clippy::collapsible_if -A clippy::module_inception
