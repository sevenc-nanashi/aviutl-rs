# aviutl-rs / AviUtl Plugin SDK for Rust

aviutl-rs は、AviUtl のプラグインを Rust で開発するためのライブラリです。

> [!WARNING]
> このライブラリは、まだ開発中です。API が大きく変更される可能性があります。

## 対応状況

- [x] output
- [ ] filter
- [ ] input
- [ ] color
- <s>[ ] language</s>（それ Rust である必要ある？）

## 使い方

プラグインの種類に応じた feature を有効にしてください。

```toml
[dependencies.aviutl]
version = "0.1"
features = ["output"]
```

使い方については、[examples](examples) を参照してください。

## ライセンス

MIT License で公開しています。詳しくは [LICENSE](LICENSE) を参照してください。
