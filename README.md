# PuyoPuyo Simulator

Rust + macroquad で書かれたぷよぷよシミュレータ。

## Development

```sh
cargo run               # ネイティブで起動
just wasm-serve         # WASM ビルド + 監視 + http://localhost:4000 で配信
```

`just` でレシピ一覧を表示。

## Documentation

- [docs/assets.md](docs/assets.md) — フォント・画像・音声の管理
- [docs/deployment.md](docs/deployment.md) — GitHub Pages へのデプロイ
- [docs/macroquad-guide.md](docs/macroquad-guide.md) — macroquad API リファレンス（メモ）
- [docs/egui-macroquad-guide.md](docs/egui-macroquad-guide.md) — egui-macroquad の使い方
