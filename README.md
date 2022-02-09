# adconverter
![Rust workflow](https://github.com/Hayashi-Yudai/adconverter/actions/workflows/rust.yml/badge.svg)

Turtle工業製16bits A/Dコンバータを動かすためのプログラム。
[THz-TDS 計測システム](https://github.com/Hayashi-Yudai/scan_system)に組み込んで使うことを前提として作成している。

## Links

- [Turtle工業 TUSB-0216ADMZ](https://www.turtle-ind.co.jp/products/ad-converters/tusb-0216admz/)
- [TUSB-0216ADMZ取扱説明書](https://www.turtle-ind.co.jp/download/manual/)
- [TUSB-0216ADMZドライバ (for Windows only)](https://www.turtle-ind.co.jp/download/win7_8_10/)

## Requirements
- Rust 1.59.0
- Windows 10

## How to use

```bash
cargo build --features release --release
```

でビルドすると `target/release/`いかに`adconverter.dll`が生成されるのでそれを使う。

このライブラリが外部に向けて用意しているのは以下の関数。

```rust
fn open(id: c_short);  // open the device
fn close(id: c_short);  // close the device
fn set_clock(id: c_short, clock_time: c_int, sel: c_uchar);
fn run(id: c_short, seconds: u64);
```

上の4つに関してはTurtle工業の製品のマニュアルを参照。`run`メソッドでは指定した時間(seconds)だけA/Dコンバータでデータを取り込んでデータを外部にpostする。
`run` メソッドを使う際には内部で `open`, `close`, `set_clock`を実行しているのでユーザーが明示的に実行する必要はない。
