# デプロイ

GitHub Pages への WASM デプロイ手順。

## 公開 URL

```
https://<github-username>.github.io/puyopuyo-simulator/
```

## 初回セットアップ（1 回のみ）

1. リポジトリの **Settings → Pages** を開く
2. **Source** を **GitHub Actions** に変更
3. 保存

この設定をしないと `actions/deploy-pages` が失敗する。

## 通常のデプロイ

`main` ブランチに push するだけ。
[.github/workflows/deploy.yml](../.github/workflows/deploy.yml) が以下を自動実行:

1. Rust toolchain + wasm32 ターゲットをセットアップ
2. `taiki-e/install-action@just` で just インストール
3. `just wasm-build` で `dist/` を生成
4. `actions/upload-pages-artifact` で `dist/` を artifact 化
5. `actions/deploy-pages` で公開

初回ビルドは依存クレートのコンパイルで 3〜5 分。
2 回目以降は `Swatinem/rust-cache` が効いて 1 分程度。

## 手動トリガー

`workflow_dispatch` イベントを設定済みなので、
GitHub の **Actions** タブから手動でも実行可能。

## ローカルでの動作確認

```sh
just wasm-serve         # → http://localhost:4000
```

`just wasm-serve` は build + watch + 配信を一括起動。
ファイル変更で自動再ビルドされる。ブラウザは手動リロード。

## トラブルシューティング

### デプロイは成功するが 404

- Pages の Source 設定を再確認（GitHub Actions になっているか）
- リポジトリが Public か、Private なら Pro / Team プランか確認

### audio が出ない

ブラウザの autoplay 制限。タイトル画面で何かキーを押すまで音は出ない仕様。
詳しくは [docs/assets.md](assets.md) の audio セクション。

### localStorage に何も保存されない

`web/quad-storage.js` と `web/sapp_jsutils.js` が `dist/` にコピーされているか確認。
これらは `just wasm-build` で自動コピーされるが、もし手動で何かする場合は必須。

## 他のプラットフォームへの展開

| サービス | 備考 |
|---|---|
| **Cloudflare Pages** | ビルド環境に Rust が無いので、build command で rustup 自体のインストールから書く必要あり |
| **Netlify** | 同上 + `netlify.toml` で WASM の Content-Type を明示するのが安全 |
| **itch.io** | `dist/` を zip 化してアップロード。CI 不要 |

公開先を変える時は `dist/` をそのまま静的ホスティングに置けばどこでも動く。
