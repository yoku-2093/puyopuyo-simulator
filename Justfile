# 既定タスク: 利用可能なレシピを表示
default:
    @just --list

# WASM をビルドして dist/ にまとめる
wasm-build:
    @rustup target list --installed | grep -q wasm32-unknown-unknown || rustup target add wasm32-unknown-unknown
    cargo build --release --target wasm32-unknown-unknown
    rm -rf dist
    mkdir -p dist
    cp target/wasm32-unknown-unknown/release/puyopuyo-simulator.wasm dist/
    cp web/index.html dist/
    cp -r assets dist/
    rm -f dist/assets/fonts/NotoSansJP-VariableFont_wght.ttf
    @echo "Build complete: dist/"

# WASM をビルドしてローカルサーバで配信 (http://localhost:4000)
wasm-serve port="4000": wasm-build
    cd dist && python3 -m http.server {{port}}

# dist/ を削除
wasm-clean:
    rm -rf dist
