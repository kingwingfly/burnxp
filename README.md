WIP

# About the Project

This project is one of the components of `the digital me` project (WIP),
which aims to clone oneself to some extent.

This component `burn_xp` is for cloning one's `Sexual Preferences`, also known as `XP`(性癖) in Chinese.

Based on [burn](https://github.com/tracel-ai/burn), `burn_xp score` can train a model to score pictures
according to owner's sexual preferences.

Based on [ratatui](https://github.com/ratatui/ratatui), `burn_xp tagger` can aid in tagging pictures by
iteractively comparing the degree of compliance with sexual preferences.

# Usage

**Cuda 12.x should be installed** for non-macOS users.

This tool depends on `libtorch` to accelerate, please set it up with provided `setup` scripts.

## 1. Use compiled release

You can download the `score` and `tagger` in the [release page](https://github.com/kingwingfly/burnxp/releases).

```sh
run/setup.xx
run/score.xx
./tagger.xx
# xx is the suffix of executable file based on your OS
```

## 2. Compile yourself

```sh
git clone git@github.com:kingwingfly/burnxp.git
scripts/setup_<your_os>
scripts/xxx_<your_os>
```

# Note

The `tagger` works well in `iTerm2` while maybe not in other terminals (like `Warp`).

A `Tauri` version will be under development.

# License

MIT LICENSE
