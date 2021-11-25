use std::os::raw::{c_int, c_short, c_uchar, c_uint};

pub mod interface;

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

/// TUSB16ADのドライバに定義されいるMicrosoft Visual Cインターフェース群
#[cfg(feature = "release")]
#[link(name = "TUSB16AD", kind = "dylib")]
#[allow(dead_code)]
extern "C" {
    fn TUSB0216AD_Device_Open(id: c_short) -> c_short;
    fn TUSB0216AD_Device_Close(id: c_short);
    /// 指定IDのデバイスのデジタル入力ポートの入力値を読み取りDataに格納する
    fn TUSB0216AD_DIO_In(id: c_short, Data: *mut c_uchar) -> c_short;
    /// 指定IDのデバイスのデジタル入力ポートの出力値を読み取りDataに格納する
    fn TUSB0216AD_DIO_Out(id: c_short, Data: c_uchar) -> c_short;
    /// 指定IDのデバイスのデジタル入力ポートの出力値を確認する
    fn TUSB0216AD_DIO_Chk(id: c_short, Data: *mut c_uchar) -> c_short;
    /// 指定IDのデバイスのアナログ入力電圧をデジタル変換して取得
    /// channel1, 2を１回ずつ変換
    /// 連続測定時には使用不可
    fn TUSB0216AD_Ad_Single(id: c_short, Data: *mut c_int) -> c_short;
    /// 連続測定を開始
    /// ch: 0 (1chのみ)、1(2chのみ)、2(1, 2ch同時)
    /// PreLen: プレトリガ長
    /// TrigType: トリガ種類、0: 内部トリガ、1: 外部デジタル、2: アナログ立ち上がり、3: アナログ立下り
    /// TrigCh: アナログトリガのトリガチャネル、0: ch1, 1: ch2
    fn TUSB0216AD_Start(
        id: c_short,
        ch: c_uchar,
        PreLen: c_int,
        TrigType: c_uchar,
        TrgCh: c_uchar,
    ) -> c_short;
    /// 連続取り込み停止
    fn TUSB0216AD_Stop(id: c_short) -> c_short;
    /// 連続取り込みの状態確認
    /// status: 0 or 2: 停止中、1: トリガ待ち、3: トリガ後変換中
    /// overflow: overflow状態, [ch1, ch2]という構造、0: overflowなし, 1: overflow
    /// datalen: 取り込み済みデータ数、[ch1, ch2]という構造
    fn TUSB0216AD_Ad_Status(
        id: c_short,
        status: *mut c_uchar,
        overflow: *mut c_uchar,
        datalen: *mut c_uint,
    ) -> c_short;
    /// 取り込み済みデータを取得
    /// data: 取り込みデータの格納先のポインタ
    /// datalen: 取り込み要求長。1~262144、戻るときには実際に取得された数が入っている
    fn TUSB0216AD_Ad_Data(
        id: c_short,
        ch: c_uchar,
        data: *mut c_int,
        datalen: *mut c_uint,
    ) -> c_short;
    /// クロック時間の設定
    /// ClkTime: 内部クロック周期設定、500 ~ 2147483647。クロック周期 = ClkTime * 20 ns
    /// sel: クロックソース、0: 内部クロック、1: 外部クロック
    fn TUSB0216AD_AdClk_Set(id: c_short, ClkTime: c_int, sel: c_uchar) -> c_short;
    /// 連続サンプリング時のアナログトリガ基準レベルの設定
    /// level: アナログ立ち上がり、下がり時の基準トリガ, 1 ~ 65534
    /// hys: ノイズ除去レベル。0 ~ 660でノイズより十分大きく信号振幅より小さな値
    fn TUSB0216AD_Level_Set(id: c_short, level: c_int, hys: c_short) -> c_short;
    /// 入力レンジの設定
    /// type1: ch1のレンジ設定
    /// type2: ch2のレンジ設定
    /// 0: +/-10V, 1: +/-5V, 2: +/-2.5V, 3: +/-1.25V, 4: 10V, 5: 5V, 6: 2.5V
    fn TUSB0216AD_Input_Set(id: c_short, type1: c_uchar, type2: c_uchar) -> c_short;
    /// 入力レンジの確認
    /// type1, type2 にはそれぞれのチャネルでのレンジの番号が入る
    /// 返り値はエラーコード
    fn TUSB0216AD_Input_Check(id: c_short, type1: *mut c_uchar, type2: *mut c_uchar) -> c_short;
    /// ソフトウェアトリガを掛ける
    fn TUSB0216AD_Trigger(id: c_short) -> c_short;
}
