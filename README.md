# typer

A minimal terminal typing speed tester written in Rust.

![demo](https://raw.githubusercontent.com/ishtiaque-f/typer/demo.png)

## features

- live WPM counter as you type
- character-by-character color feedback (green = correct, red = wrong)
- progress bar
- accuracy tracking
- results screen with color-coded performance
- 15 built-in prompts, randomly selected

## install

### from source

```bash
git clone https://github.com/your-username/typer
cd typer
cargo build --release
./target/release/typer
```

### copy binary to PATH

```bash
sudo cp target/release/typer /usr/local/bin/
```

then just run `typer` from anywhere.

## usage

just start typing the prompt shown on screen. the timer starts on your first keypress.

| key        | action              |
|------------|---------------------|
| `ctrl+r`   | restart current prompt |
| `ctrl+c`   | quit                |
| `r`        | retry (on results screen) |
| `n`        | new prompt (on results screen) |
| `q`        | quit (on results screen) |

## wpm color coding

| color  | speed       |
|--------|-------------|
| 🟢 green  | 60+ wpm     |
| 🟡 yellow | 35–59 wpm   |
| 🔴 red    | below 35 wpm |

## built with

- [crossterm](https://github.com/crossterm-rs/crossterm) — cross-platform terminal manipulation
- [rand](https://github.com/rust-random/rand) — random prompt selection
