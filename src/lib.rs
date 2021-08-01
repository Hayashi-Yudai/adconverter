#[link(name = "TUSB16AD", kind = "dylib")]
#[allow(dead_code)]
extern "C" {
    fn TUSB0216AD_Device_Open(id: i32) -> i32;
    fn TUSB0216AD_Device_Close(id: i32);
    /// 指定IDのデバイスのデジタル入力ポートの入力値を読み取りDataに格納する
    fn TUSB0216AD_DIO_In(id: i32, Data: &u8) -> i32;
    /// 指定IDのデバイスのデジタル入力ポートの出力値を読み取りDataに格納する
    fn TUSB0216AD_DIO_Out(id: i32, Data: &u8) -> i32;
    /// 指定IDのデバイスのデジタル入力ポートの出力値を確認する
    fn TUSB0216AD_DIO_Chk(id: i32, Data: &u8) -> i32;
    /// 指定IDのデバイスのアナログ入力電圧をデジタル変換して取得
    /// channel1, 2を１回ずつ変換
    /// 連続測定時には使用不可
    fn TUSB0216AD_Ad_Single(id: i32, Data: &i32) -> i32;
    /// 連続測定を開始
    /// ch: 0 (1chのみ)、1(2chのみ)、2(1, 2ch同時)
    /// PreLen: プレトリガ長
    /// TrigType: トリガ種類、0: 内部トリガ、1: 外部デジタル、2: アナログ立ち上がり、3: アナログ立下り
    /// TrigCh: アナログトリガのトリガチャネル、0: ch1, 1: ch2
    fn TUSB0216AD_Start(id: i32, ch: &u8, PreLen: i32, TrigType: u8, TrgCh: u8) -> i32;
    /// 連続取り込み停止
    fn TUSB0216AD_Stop(id: i32) -> i32;
    /// 連続取り込みの状態確認
    /// status: 0 or 2: 停止中、1: トリガ待ち、3: トリガ後変換中
    /// overflow: overflow状態, [ch1, ch2]という構造、0: overflowなし, 1: overflow
    /// datalen: 取り込み済みデータ数、[ch1, ch2]という構造
    fn TUSB0216AD_Ad_Status(id: i32, status: &u8, overflow: &u8, datalen: &u16) -> i32;
    /// 取り込み済みデータを取得
    /// data: 取り込みデータの格納先のポインタ
    /// datalen: 取り込み要求長。1~262144、戻るときには実際に取得された数が入っている
    fn TUSB0216AD_Ad_Data(id: i32, ch: u8, data: &i32, datalen: u32) -> i32;
    /// クロック時間の設定
    /// ClkTime: 内部クロック周期設定、500 ~ 2147483647。クロック周期 = ClkTime * 20 ns
    /// sel: クロックソース、0: 内部クロック、1: 外部クロック
    fn TUSB0216AD_AdClk_Set(id: i32, ClkTime: i32, sel: u8) -> i32;
    /// 連続サンプリング時のアナログトリガ基準レベルの設定
    /// level: アナログ立ち上がり、下がり時の基準トリガ, 1 ~ 65534
    /// hys: ノイズ除去レベル。0 ~ 660でノイズより十分大きく信号振幅より小さな値
    fn TUSB0216AD_Level_Set(id: i32, level: i32, hys: i32) -> i32;
    /// 入力レンジの設定
    /// type1: ch1のレンジ設定
    /// type2: ch2のレンジ設定
    /// 0: +/-10V, 1: +/-5V, 2: +/-2.5V, 3: +/-1.25V, 4: 10V, 5: 5V, 6: 2.5V
    fn TUSB0216AD_Input_Set(id: i32, type1: u8, type2: u8) -> i32;
    /// 入力レンジの確認
    fn TUSB0216AD_Input_Check(id: i32, type1: &u8, type2: &u8) -> i32;
    /// ソフトウェアトリガを掛ける
    fn TUSB0216AD_Trigger(id: i32) -> i32;
}

/*
enum DevError {
    NOERROR = 0,
    INVALIDID = 1,
    INVALIDDRIVER = 2,
    ALREADYOPENEDDEV = 3,
    TOOMANYDEV = 4,
    FAILTOOPENDEV = 5,
    DEVNOTFOUND = 6,
    INVALIDPARAM = 8,
    USBERR = 9,
    SEQUENTIALREAD = 11,
    OTHERERR = 99,
}
*/

fn parse_error(e: i32) {
    match e {
        0 => println!("No error"),
        1 => println!("Invalid ID"),
        2 => println!("Invalid Driver"),
        3 => println!("Device already opened"),
        4 => println!("Too many devices"),
        5 => println!("Failed to open device"),
        6 => println!("Device not found"),
        8 => println!("Parameters are invalid"),
        9 => println!("USB connection error"),
        11 => println!("Sequential reading"),
        99 => println!("Other error"),
        _ => println!("Other error"),
    }

    if e != 0 {
        panic!("Fatal error occurred");
    }
}

#[no_mangle]
pub extern "C" fn device_test() {
    let e: i32;
    unsafe {
        e = TUSB0216AD_Device_Open(1);
    }
    if e != 0 {
        println!("Error code: {}", e);
        parse_error(e);
    }

    unsafe {
        TUSB0216AD_Device_Close(1);
    }
}
