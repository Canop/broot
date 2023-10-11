
**broot** is developed by **Denys Séguret**, also known as [Canop](https://github.com/Canop) or [dystroy](https://dystroy.org).

Major updates are announced on my Mastodon account: [@dystroy@mastodon.dystroy.org](https://mastodon.dystroy.org/@dystroy).

# Sponsorship

**broot** is free for all uses.

If it helps your company make money, consider helping me find time to add features and to develop new free open-source software.

<div class=sponsorship>
<script src="https://liberapay.com/dystroy/widgets/button.js"></script>
<noscript><a href="https://liberapay.com/dystroy/donate"><img alt="Donate using Liberapay" src="https://liberapay.com/assets/widgets/donate.svg"></a></noscript>

<iframe src="https://github.com/sponsors/Canop/button" title="Sponsor Canop" height="35" width="114" style="border: 0; border-radius: 6px;"></iframe>
</div>

# Discuss Broot in a chat room

The best place to chat about broot, to talk about features or bugs, is the Miaou chat.

There's a dedicated room:

[![Chat on Miaou](https://miaou.dystroy.org/static/shields/room-en.svg?v=1)](https://miaou.dystroy.org/3490?broot) **broot**

If you're French speaking, you might prefer to directly come where other French speaking programmers hang:

[![Chat on Miaou](https://miaou.dystroy.org/static/shields/room-fr.svg?v=1)](https://miaou.dystroy.org/3) **Code & Croissants**

Don't hesitate to come if you have a question.

# Issues

We use [GitHub's issue manager](https://github.com/Canop/broot/issues).

Before posting a new issue, check your problem hasn't already been raised and in case of doubt **please come first discuss it on the chat**.

# Your wishes

[Issues](https://github.com/Canop/broot/issues) is also where I test new ideas. If you're interested in the directions broot takes, **please come and vote on issues**, or maybe comment. This would help me prioritize developments: if nobody's interested in a feature I'm not sure I want, I'll do something else.

# Log

When something looks like a bug, especially keyboard problems, I need both to know the exact configuration (OS, terminal program, mainly) and to have the log. The log can be obtained this way:

1. start broot with `BROOT_LOG=debug br`
2. do the action which seems not to properly work, and nothing else
3. quit broot
4. go to the chat (or the GitHub issue if you already made one) and paste the content of the `broot.log` file

# Benchmark

To get a precise idea of the time taken by operations in real broot use, it's often a good idea to run them with `--cmd`.

For example full text search performances can be measured (and compared to other tools) with

```
time broot -c "c/memmap;:pt" ~/code
```

# Contribute

**Broot** is written in [Rust](https://www.rust-lang.org/). The current focus is linux+mac but I try to support Windows too (use a modern terminal like the [new MS one](https://github.com/microsoft/terminal)).

If you think you might help, as a tester or coder, you're welcome, but please read [Contributing to my FOSS projects](https://dystroy.org/blog/contributing/) before starting a PR.

# This documentation...

... needs your help too.

Tell us what seems to be unclear or missing, what tricks should be added.

And if you have interesting screenshots telling a story, they might find their way into a new page too.
