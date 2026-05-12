# PuyoPuyo Simulator

ぷよぷよのシミュレータ。実際のプレイ感に近づけつつ連鎖のシミュレーションを試せる環境を用意することで、esportsとしてのぷよぷよへの貢献を目指す。
Rust + macroquad 製。

🎮 **Play**: https://yoku-2093.github.io/puyopuyo-simulator/

<!-- 録画 / スクショを置きたい時はここ -->
<!-- ![demo](docs/demo.gif) -->

## Controls

| Key | Action |
|---|---|
| ← / → | Move |
| ↓ | Soft drop |
| Z | Rotate left |
| X | Rotate right |
| Enter / Space | Start |
| S | Settings |
| Esc | Back to title |

## Run locally

```sh
cargo run               # Desktop
just wasm-serve         # Web (http://localhost:4000)
```

`just` でレシピ一覧表示。

## Tech stack

- [macroquad](https://macroquad.rs/) — game framework
- [egui-macroquad](https://github.com/optozorax/egui-macroquad) — settings UI
- [quad-storage](https://github.com/optozorax/quad-storage) — local persistence
