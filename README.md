# adconverter
Turtle工業製16bits A/Dコンバータを動かすためのプログラム。
[THz-TDS 計測システム](https://github.com/Hayashi-Yudai/scan_system)に組み込んで使うことを前提として作成している。

## Links

- [Turtle工業 TUSB-0216ADMZ](https://www.turtle-ind.co.jp/products/ad-converters/tusb-0216admz/)
- [TUSB-0216ADMZ取扱説明書](https://www.turtle-ind.co.jp/download/manual/)
- [TUSB-0216ADMZドライバ (for Windows only)](https://www.turtle-ind.co.jp/download/win7_8_10/)

## Requirements
- Rust 1.55.0
- Windows 10

## How to use

```bash
cargo build --features release --release
```

でビルドすると `target/release/`いかに`adconverter.dll`が生成されるのでそれを使う。

このライブラリが外部に向けて用意しているのは以下の関数。

```rust
fn open(id: i32) -> i32;
fn close(id: i32) -> i32;
fn set_clock(id: i32, clock_time: i32, sel: u8) -> i32;
fn run(id: i32, seconds: u64);
```

上の３つに関してはTurtle工業の製品のマニュアルを参照。`run`メソッドでは指定した時間(s)A/Dコンバータでデータを取り込んでデータを外部にpostする。
