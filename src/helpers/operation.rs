use std::f64::consts::PI;

/// TUSB16ADのドライバに定義されいるMicrosoft Visual Cインターフェース群
#[link(name = "TUSB16AD", kind = "dylib")]
//#[link(name = "ad_mock.dll", kind = "dylib")] // Mock
#[allow(dead_code)]
#[cfg(feature = "release")]
extern "C" {
    pub fn TUSB0216AD_Device_Open(id: i32) -> i32;
    pub fn TUSB0216AD_Device_Close(id: i32);
    /// 指定IDのデバイスのデジタル入力ポートの入力値を読み取りDataに格納する
    pub fn TUSB0216AD_DIO_In(id: i32, Data: &u8) -> i32;
    /// 指定IDのデバイスのデジタル入力ポートの出力値を読み取りDataに格納する
    pub fn TUSB0216AD_DIO_Out(id: i32, Data: &u8) -> i32;
    /// 指定IDのデバイスのデジタル入力ポートの出力値を確認する
    pub fn TUSB0216AD_DIO_Chk(id: i32, Data: &u8) -> i32;
    /// 指定IDのデバイスのアナログ入力電圧をデジタル変換して取得
    /// channel1, 2を１回ずつ変換
    /// 連続測定時には使用不可
    pub fn TUSB0216AD_Ad_Single(id: i32, Data: &i32) -> i32;
    /// 連続測定を開始
    /// ch: 0 (1chのみ)、1(2chのみ)、2(1, 2ch同時)
    /// PreLen: プレトリガ長
    /// TrigType: トリガ種類、0: 内部トリガ、1: 外部デジタル、2: アナログ立ち上がり、3: アナログ立下り
    /// TrigCh: アナログトリガのトリガチャネル、0: ch1, 1: ch2
    pub fn TUSB0216AD_Start(id: i32, ch: &u8, PreLen: i32, TrigType: u8, TrgCh: u8) -> i32;
    /// 連続取り込み停止
    pub fn TUSB0216AD_Stop(id: i32) -> i32;
    /// 連続取り込みの状態確認
    /// status: 0 or 2: 停止中、1: トリガ待ち、3: トリガ後変換中
    /// overflow: overflow状態, [ch1, ch2]という構造、0: overflowなし, 1: overflow
    /// datalen: 取り込み済みデータ数、[ch1, ch2]という構造
    pub fn TUSB0216AD_Ad_Status(id: i32, status: &u8, overflow: &u8, datalen: &u16) -> i32;
    /// 取り込み済みデータを取得
    /// data: 取り込みデータの格納先のポインタ
    /// datalen: 取り込み要求長。1~262144、戻るときには実際に取得された数が入っている
    pub fn TUSB0216AD_Ad_Data(id: i32, ch: u8, data: *mut i32, datalen: *mut u32) -> i32;
    /// クロック時間の設定
    /// ClkTime: 内部クロック周期設定、500 ~ 2147483647。クロック周期 = ClkTime * 20 ns
    /// sel: クロックソース、0: 内部クロック、1: 外部クロック
    pub fn TUSB0216AD_AdClk_Set(id: i32, ClkTime: i32, sel: u8) -> i32;
    /// 連続サンプリング時のアナログトリガ基準レベルの設定
    /// level: アナログ立ち上がり、下がり時の基準トリガ, 1 ~ 65534
    /// hys: ノイズ除去レベル。0 ~ 660でノイズより十分大きく信号振幅より小さな値
    pub fn TUSB0216AD_Level_Set(id: i32, level: i32, hys: i32) -> i32;
    /// 入力レンジの設定
    /// type1: ch1のレンジ設定
    /// type2: ch2のレンジ設定
    /// 0: +/-10V, 1: +/-5V, 2: +/-2.5V, 3: +/-1.25V, 4: 10V, 5: 5V, 6: 2.5V
    pub fn TUSB0216AD_Input_Set(id: i32, type1: u8, type2: u8) -> i32;
    /// 入力レンジの確認
    /// type1, type2 にはそれぞれのチャネルでのレンジの番号が入る
    /// 返り値はエラーコード
    pub fn TUSB0216AD_Input_Check(id: i32, type1: *mut u8, type2: *mut u8) -> i32;
    /// ソフトウェアトリガを掛ける
    pub fn TUSB0216AD_Trigger(id: i32) -> i32;
}

// Debug用 mock
#[cfg(feature = "debug")]
#[allow(non_snake_case)]
pub unsafe fn TUSB0216AD_Device_Open(id: i32) -> i32 {
    match id {
        1 => 0,
        _ => 5,
    }
}

#[cfg(feature = "debug")]
#[allow(non_snake_case)]
pub unsafe fn TUSB0216AD_Device_Close(id: i32) -> i32 {
    match id {
        1 => 0,
        _ => 5,
    }
}

#[cfg(feature = "debug")]
#[allow(non_snake_case)]
pub unsafe fn TUSB0216AD_AdClk_Set(id: i32, clock_time: i32, sel: u8) -> i32 {
    if id != 1 {
        return 5;
    }

    if clock_time < 500 {
        return 8;
    }

    if sel != 0 && sel != 1 {
        return 8;
    }

    0
}

#[cfg(feature = "debug")]
#[allow(non_snake_case)]
pub unsafe fn TUSB0216AD_Start(id: i32, ch: &u8, _PreLen: i32, TrigType: u8, TrgCh: u8) -> i32 {
    if id != 1 || *ch > 2 || TrigType > 3 || TrgCh > 1 {
        return 5;
    }

    0
}

#[cfg(feature = "debug")]
#[allow(non_snake_case)]
pub unsafe fn TUSB0216AD_Stop(id: i32) -> i32 {
    if id != 1 {
        return 5;
    }

    0
}

#[cfg(feature = "debug")]
#[allow(non_snake_case)]
pub unsafe fn TUSB0216AD_Input_Set(id: i32, type1: u8, type2: u8) -> i32 {
    if id != 1 || type1 > 6 || type2 > 6 {
        return 5;
    }
    0
}

#[cfg(feature = "debug")]
#[allow(non_snake_case)]
pub unsafe fn TUSB0216AD_Input_Check(id: i32, type1: *mut u8, type2: *mut u8) -> i32 {
    if id != 1 {
        return 5;
    }
    *type1 = 0;
    *type2 = 0;

    0
}

#[cfg(feature = "debug")]
#[allow(non_snake_case)]
pub unsafe fn TUSB0216AD_Trigger(id: i32) -> i32 {
    if id != 1 {
        return 5;
    }
    0
}

#[cfg(feature = "debug")]
#[allow(non_snake_case)]
pub unsafe fn TUSB0216AD_Ad_Data(id: i32, ch: u8, data: *mut i32, datalen: *mut u32) -> i32 {
    if id != 1 {
        return 5;
    }

    if ch != 0 && ch != 1 {
        return 8;
    }
    *datalen = 10000;
    for i in 0..10000 {
        *data.offset(i) =
            (2f32.powf(15.0) * ((2e-4 * 2.0 * PI as f32 * i as f32).sin() + 1.0)) as i32;
    }
    0
}
