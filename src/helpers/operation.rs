#[cfg(not(feature = "release"))]
use std::f64::consts::PI;

use crate::helpers::helper;
use std::os::raw::{c_int, c_short, c_uchar, c_uint};

/// TUSB16ADのドライバに定義されいるMicrosoft Visual Cインターフェース群
#[link(name = "TUSB16AD", kind = "dylib")]
#[allow(dead_code)]
#[cfg(feature = "release")]
extern "C" {
    pub fn TUSB0216AD_Device_Open(id: c_short) -> c_short;
    pub fn TUSB0216AD_Device_Close(id: c_short);
    /// 指定IDのデバイスのデジタル入力ポートの入力値を読み取りDataに格納する
    pub fn TUSB0216AD_DIO_In(id: c_short, Data: *mut c_uchar) -> c_short;
    /// 指定IDのデバイスのデジタル入力ポートの出力値を読み取りDataに格納する
    pub fn TUSB0216AD_DIO_Out(id: c_short, Data: c_uchar) -> c_short;
    /// 指定IDのデバイスのデジタル入力ポートの出力値を確認する
    pub fn TUSB0216AD_DIO_Chk(id: c_short, Data: *mut c_uchar) -> c_short;
    /// 指定IDのデバイスのアナログ入力電圧をデジタル変換して取得
    /// channel1, 2を１回ずつ変換
    /// 連続測定時には使用不可
    pub fn TUSB0216AD_Ad_Single(id: c_short, Data: *mut c_int) -> c_short;
    /// 連続測定を開始
    /// ch: 0 (1chのみ)、1(2chのみ)、2(1, 2ch同時)
    /// PreLen: プレトリガ長
    /// TrigType: トリガ種類、0: 内部トリガ、1: 外部デジタル、2: アナログ立ち上がり、3: アナログ立下り
    /// TrigCh: アナログトリガのトリガチャネル、0: ch1, 1: ch2
    pub fn TUSB0216AD_Start(
        id: c_short,
        ch: c_uchar,
        PreLen: c_int,
        TrigType: c_uchar,
        TrgCh: c_uchar,
    ) -> c_short;
    /// 連続取り込み停止
    pub fn TUSB0216AD_Stop(id: c_short) -> c_short;
    /// 連続取り込みの状態確認
    /// status: 0 or 2: 停止中、1: トリガ待ち、3: トリガ後変換中
    /// overflow: overflow状態, [ch1, ch2]という構造、0: overflowなし, 1: overflow
    /// datalen: 取り込み済みデータ数、[ch1, ch2]という構造
    pub fn TUSB0216AD_Ad_Status(
        id: c_short,
        status: *mut c_uchar,
        overflow: *mut c_uchar,
        datalen: *mut c_uint,
    ) -> c_short;
    /// 取り込み済みデータを取得
    /// data: 取り込みデータの格納先のポインタ
    /// datalen: 取り込み要求長。1~262144、戻るときには実際に取得された数が入っている
    pub fn TUSB0216AD_Ad_Data(
        id: c_short,
        ch: c_uchar,
        data: *mut c_int,
        datalen: *mut c_uint,
    ) -> c_short;
    /// クロック時間の設定
    /// ClkTime: 内部クロック周期設定、500 ~ 2147483647。クロック周期 = ClkTime * 20 ns
    /// sel: クロックソース、0: 内部クロック、1: 外部クロック
    pub fn TUSB0216AD_AdClk_Set(id: c_short, ClkTime: c_int, sel: c_uchar) -> c_short;
    /// 連続サンプリング時のアナログトリガ基準レベルの設定
    /// level: アナログ立ち上がり、下がり時の基準トリガ, 1 ~ 65534
    /// hys: ノイズ除去レベル。0 ~ 660でノイズより十分大きく信号振幅より小さな値
    pub fn TUSB0216AD_Level_Set(id: c_short, level: c_int, hys: c_short) -> c_short;
    /// 入力レンジの設定
    /// type1: ch1のレンジ設定
    /// type2: ch2のレンジ設定
    /// 0: +/-10V, 1: +/-5V, 2: +/-2.5V, 3: +/-1.25V, 4: 10V, 5: 5V, 6: 2.5V
    pub fn TUSB0216AD_Input_Set(id: c_short, type1: c_uchar, type2: c_uchar) -> c_short;
    /// 入力レンジの確認
    /// type1, type2 にはそれぞれのチャネルでのレンジの番号が入る
    /// 返り値はエラーコード
    pub fn TUSB0216AD_Input_Check(id: c_short, type1: *mut c_uchar, type2: *mut c_uchar)
        -> c_short;
    /// ソフトウェアトリガを掛ける
    pub fn TUSB0216AD_Trigger(id: c_short) -> c_short;
}

pub struct DeviceStatus {
    pub status: c_uchar,
    pub ch1_datalen: c_uint,
    pub ch2_datalen: c_uint,
}

impl DeviceStatus {
    fn new(status: c_uchar, ch1_datalen: c_uint, ch2_datalen: c_uint) -> Self {
        DeviceStatus {
            status,
            ch1_datalen,
            ch2_datalen,
        }
    }
}

/// Open device with specified ID
pub fn open(id: c_short) {
    #[cfg(feature = "release")]
    {
        let error: c_short;
        unsafe {
            error = TUSB0216AD_Device_Open(id);
        }
        helper::parse_error(error, "TUSB0216AD_Device_Open");
    }

    #[cfg(not(feature = "release"))]
    {
        let error: c_short;
        match id {
            1 => error = 0,
            _ => error = 5,
        }
        helper::parse_error(error, "TUSB0216AD_Device_Open");
    }
}

/// Close the connection with the device
pub fn close(id: c_short) {
    #[cfg(feature = "release")]
    {
        unsafe {
            TUSB0216AD_Device_Close(id);
        }
    }
}

#[allow(dead_code)]
pub fn single_data(id: c_short, data: *mut c_int) {
    #[cfg(feature = "release")]
    {
        let error: c_short;
        unsafe {
            error = TUSB0216AD_Ad_Single(id, data);
        }

        helper::parse_error(error, "TUSB0216AD_Ad_Single");
    }
}

pub fn start(id: c_short, ch: c_uchar, prelen: c_int, trig_type: c_uchar, trig_ch: c_uchar) {
    let error: c_short;

    #[cfg(feature = "release")]
    {
        unsafe {
            error = TUSB0216AD_Start(id, ch, prelen, trig_type, trig_ch);
        }
    }
    #[cfg(not(feature = "release"))]
    {
        if id != 1 || ch > 2 || prelen < 0 || trig_type > 3 || trig_ch > 1 {
            error = 5;
        } else {
            error = 0;
        }
    }
    helper::parse_error(error, "TUSB0216AD_Start");
}

pub fn stop(id: c_short) {
    let error: c_short;
    #[cfg(feature = "release")]
    {
        unsafe {
            error = TUSB0216AD_Stop(id);
        }
    }
    #[cfg(not(feature = "release"))]
    {
        if id != 1 {
            error = 5;
        } else {
            error = 0;
        }
    }
    helper::parse_error(error, "TUSB0216AD_Stop");
}

/// Show the device status
///
/// * verbose: bool
///     if true, it prints the status on the screen
pub fn status(verbose: bool) -> DeviceStatus {
    let mut status = 1 as u8;
    let mut overflow = [0, 0];
    let mut datalen = [0, 0];
    #[cfg(feature = "release")]
    {
        unsafe {
            TUSB0216AD_Ad_Status(
                0,
                &mut status as *mut u8,
                overflow.as_mut_ptr(),
                datalen.as_mut_ptr(),
            );
        }
    }

    match verbose {
        true => {
            println!("============");
            println!("Status: {}", status);
            println!("Overflow: {:?}", overflow);
            println!("DataLen: {:?}", datalen);
            println!("============");
        }
        false => {}
    }

    DeviceStatus::new(status, datalen[0], datalen[1])
}

pub fn takeout_data(id: c_short, ch: c_uchar, data: *mut c_int, length: *mut c_uint) {
    let mut error: c_short;

    #[cfg(feature = "release")]
    {
        unsafe {
            error = TUSB0216AD_Ad_Data(id, ch, data, length);
        }
    }
    #[cfg(not(feature = "release"))]
    {
        if id != 1 {
            error = 5;
            helper::parse_error(error, "TUSB0216AD_Ad_Data");
        }
        unsafe {
            if ch != 0 && ch != 1 {
                error = 8;
            } else {
                *length = 10000;
                for i in 0..10000 {
                    *data.offset(i) = (2f32.powf(15.0)
                        * ((2e-4 * 2.0 * PI as f32 * i as f32).sin() + 1.0))
                        as i32;
                }
                error = 0;
            }
        }
    }
    helper::parse_error(error, "TUSB0216AD_Ad_Data");
}

pub fn set_clock(id: c_short, clock_time: c_int, sel: c_uchar) {
    let mut error: c_short;
    #[cfg(feature = "release")]
    {
        unsafe {
            error = TUSB0216AD_AdClk_Set(id, clock_time, sel);
        }
    }
    #[cfg(not(feature = "release"))]
    {
        error = 0;
        if id != 1 {
            error = 5;
        }

        if clock_time < 500 {
            error = 8;
        }

        if sel != 0 && sel != 1 {
            error = 8;
        }
    }
    helper::parse_error(error, "TUSB0216AD_AdClk_Set");
}

/// Change input range of each channel.
/// Specify the input ranges with a number.
/// 0: +/-10 V, 1: +/-5V, 2: +/-2.5 V, 3: +/-1.25V
/// 4: 10 V, 5: 5 V, 6: 2.5 V
///
/// * `id` - Device number
/// * `type1` - input range of CH1
/// * `type2` - input range of CH2
pub fn input_set(id: c_short, type1: c_uchar, type2: c_uchar) {
    let error: c_short;
    #[cfg(feature = "release")]
    {
        unsafe {
            error = TUSB0216AD_Input_Set(id, type1, type2);
        }
    }
    #[cfg(not(feature = "release"))]
    {
        if id != 1 || type1 > 6 || type2 > 6 {
            error = 5;
        } else {
            error = 0;
        }
    }
    helper::parse_error(error, "TUSB0216AD_Input_Set");
}

pub fn input_check(id: c_short, type1: *mut c_uchar, type2: *mut c_uchar) {
    let error: c_short;
    #[cfg(feature = "release")]
    {
        unsafe {
            error = TUSB0216AD_Input_Check(id, type1, type2);
        }
    }
    #[cfg(not(feature = "release"))]
    {
        unsafe {
            if id != 1 {
                error = 5;
            } else {
                error = 0;
            }
            *type1 = 0;
            *type2 = 0;
        }
    }
    helper::parse_error(error, "TUSB0216AD_Input_Check");
}

pub fn trigger(id: c_short) {
    let error: c_short;
    #[cfg(feature = "release")]
    {
        unsafe {
            error = TUSB0216AD_Trigger(id);
        }
    }
    #[cfg(not(feature = "release"))]
    {
        if id != 1 {
            error = 5;
        } else {
            error = 0;
        }
    }
    helper::parse_error(error, "TUSB0216AD_Trigger");
}
