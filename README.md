# About the Project

This project is one of the components of `the digital me` (WIP),
which aims to clone oneself to some extent.

This component `burnxp` is for cloning one's `Aesthetics Preferences`, also known as `XP`(性癖) in Chinese.

Based on [burn](https://github.com/tracel-ai/burn), `burnxp score` can train a RNN model to score pictures
according to owner's sesthetics preferences.

Based on [ratatui](https://github.com/ratatui/ratatui), `burnxp tagger` can aid in tagging pictures by
iteractively comparing the degree of compliance with sesthetics preferences.

## Tagger

<table>
    <tr>
        <td>
            <img src="images/tagger_screenshot.png" height="260px"/><br />
            <img src="images/tagger_observe_screenshot.png" height="260px"/>
        </td>
        <td>
            <img src="images/tagger_picker_screenshot.png" height="520px"/>
        </td>
    </tr>
</table>


Features:
- Sort pictures by comparing one by one (clever data structure and algorithm are used to
guarantee total order and O(nlogn) complexity)
- L/R/U/D arrow keys for better/worse/much better/much worse
- Sort, score and group pictures in json format
- The user input will be cached so that user can continue from where he/she left off
- `tagger pick` subcommand can help pick the images to be tagged **(super fast image viewer in terminal)**
- `tagger divide` subcommand can help divide the images into train-set and valid-set
- `tagger observe` subcommand can help observe the distribution of scores

## Trainer

![train_screenshot](images/train_screenshot.png)

## Predictor

![train_screenshot](images/predict_screenshot.png)

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
scripts/setup_<your_os>.xx
# macOS only
source .venv/bin/activate

cargo build -p tagger --release
cargo build -p score --release
```

# Note

The `tagger` works well in `kitty` `iTerm2` and `wezterm` while maybe not in other terminals (like `Warp`).

For Windows, I tried my best to make it work, but Windows is just a piece of \*\*;

A `Tauri` version may be under development.

# Contributing

Please export all needed environment variables like `scripts/setup` before coding, or your IDE may not work well.

For macOS users, you need also activate python venv before coding.

# License

MIT LICENSE
