# アセット管理

## ディレクトリ構成

```
assets/
├── fonts/
│   ├── NotoSansJP.ttf                       (142KB, サブセット済み・配布対象)
│   └── NotoSansJP-VariableFont_wght.ttf     (8.7MB, サブセット元・gitignore 対象)
├── images/
│   ├── background/
│   ├── puyo/
│   └── game_over.png
├── music/
│   └── bgm.ogg
└── sounds/
    └── puyo.ogg
```

## フォント

[`render.rs`](../src/render.rs) で `include_bytes!` により WASM バイナリに直接埋め込んでいる。
これにより runtime での fetch が不要で、WASM 配布時に font ファイルを別途配信しなくて良い。

### サブセット化の方針

含める Unicode 範囲:

| 範囲 | 用途 |
|---|---|
| `U+0020-007F` | ASCII |
| `U+2190-21FF` | 矢印（← → ↓） |
| `U+3000-303F` | CJK 記号・句読点 |
| `U+3040-309F` | ひらがな |
| `U+30A0-30FF` | カタカナ |
| `U+FF00-FFEF` | 全角英数字・記号 |

### フォント再生成手順

ソースの可変フォントから wght=600 (Bold 相当) に固定 → 必要文字だけ抽出:

```sh
python3 -m fontTools.varLib.instancer \
  assets/fonts/NotoSansJP-VariableFont_wght.ttf wght=600 \
  -o /tmp/noto_static.ttf

python3 -m fontTools.subset /tmp/noto_static.ttf \
  --unicodes="U+0020-007F,U+2190-21FF,U+3000-303F,U+3040-309F,U+30A0-30FF,U+FF00-FFEF" \
  --layout-features-=kern \
  --no-hinting \
  --desubroutinize \
  --output-file=assets/fonts/NotoSansJP.ttf
```

文字種を追加したい場合は `--unicodes` の範囲に追加。

## オーディオ

`quad-snd` (macroquad 内部) は OGG Vorbis と WAV のみサポート。MP3 等は OGG に変換する必要がある。

### 音量調整

```sh
# 例: 1.5 倍に増幅
ffmpeg -i input.ogg -filter:a "volume=1.5" /tmp/tmp.wav
ffmpeg -i /tmp/tmp.wav -c:a vorbis -strict -2 -q:a 5 output.ogg
```

OGG → OGG の直接変換は libvorbis のエンコーダバグ
([assertion failure in vorbisenc.c:869](https://gitlab.xiph.org/xiph/vorbis/-/issues/2293))
を踏むことがあるので、**WAV を経由する** のが安全。

ffmpeg のビルドに `libvorbis` が無い環境では `-c:a vorbis -strict -2` で内蔵 vorbis エンコーダを使う。

## 画像

`load_texture` で読み込み。PNG 推奨。`set_filter(FilterMode::Nearest)` でドット絵のシャープさを維持。
