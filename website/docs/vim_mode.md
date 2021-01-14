
# "Vim Mode" (or "Modal Mode")

## Warnings

1. DON'T activate this mode until you really tried broot. Broot isn't a text editor and can't be confused with one. This mode may be more comfortable when you constantly jump from vim to broot but only after you understood how broot works.
1. This isn't really about a "vim mode". This is about a "modal mode" in which single letter key shortcuts are possible because the input at the bottom isn't always focused. You may devise single letter shortcuts without taking inspiration in vim
1. You may be an avid vim user, as I am, and still prefer not to use modality in broot. Starting in *command* mode means you have one more letter to type before searching, because search is done in *input* mode. And broot is search oriented and often used in very short sessions (less than 5 seconds from intent to launch to being back in the shell in the right directory or editing the right file in your favorite editor)

## Configuration

You need first to enable the "modal mode" with this line in the configuration:

```hjson
modal: true
```

(or `modal = true` at the begining of the configuration if it's in TOML)

If `modal` isn't set to `true`, the single letter shortcuts you define in configuration will be ignored (so you don't have to remove them if you don't want modality anymore).

## Usage

Broot may be in one of two modes:

1. **input** mode, with the input field at the bottom focused and received standard keys
1. **command** mode, with input not focused, and single key shortcuts enabled

In *command* mode, you'll find those keys already configured:
* `j` and `k` to go down and up
* ̀ h` and `l` to go to parent or to enter a directory

You enter *input* mode by typing one of those letters: ` ` (space), `:`, or `/`. You leave it with the `escape` key. You may add other bindings to the `:mode_input` and `:mode_command` verbs.


## Experimentation

This "vim mode" is still experimental. If you tried it, or use it, I'd like your feedback on [Miaou](https://miaou.dystroy.org/3490).
