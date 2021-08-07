use std::sync::{Arc, Mutex};
use std::{thread, time};

// #[link(name = "TUSB16AD", kind = "dylib")]
#[link(name = "ad_mock.dll", kind = "dylib")]
#[allow(dead_code)]
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
	pub fn TUSB0216AD_Input_Check(id: i32, type1: &u8, type2: &u8) -> i32;
	/// ソフトウェアトリガを掛ける
	pub fn TUSB0216AD_Trigger(id: i32) -> i32;
}

pub fn parse_error(e: i32) {
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
}

struct Data {
	x: f32,
	y: f32,
	len: u32,
}

impl Data {
	fn new(x: f32, y: f32) -> Self {
		Data { x: x, y: y, len: 1 }
	}
}

pub fn continuous_read(id: i32, seconds: u64, flag: Arc<Mutex<i8>>) {
	println!("Timer start!");
	let sleeping_time = time::Duration::from_secs(seconds);

	unsafe {
		// TODO: ステージ位置に関しては、+/-10Vが最適かわからない
		// CH1, 2ともに+/-10Vの入力を受け付ける
		// 入力が+/-10VなのはSR830の仕様
		TUSB0216AD_Input_Set(id, 0, 0);
		TUSB0216AD_Start(id, &(2 as u8), 0, 0, 0);
		TUSB0216AD_Trigger(id);
	}

	*flag.lock().unwrap() = 0; // 計測開始のフラグを立てる
	thread::sleep(sleeping_time);

	unsafe {
		TUSB0216AD_Stop(id);
	}

	*flag.lock().unwrap() = 1; // 計測終了のフラグを立てる
	println!("Timer stopped");
}

/// CH1, CH2 からのデータを取得する
pub fn get_data(id: i32, flag: Arc<Mutex<i8>>, position: &mut Vec<f32>, intensity: &mut Vec<f32>) {
	// const MAX_LENGTH: usize = 262142;
	const MAX_LENGTH: usize = 100000;
	let mut length = MAX_LENGTH as u32;

	println!("Data acquisition started");
	loop {
		if *flag.lock().unwrap() != -1 {
			break;
		}
		thread::sleep(time::Duration::from_millis(1));
	}

	let mut data1 = [0; MAX_LENGTH];
	let mut data2 = [0; MAX_LENGTH];
	let l_ptr = &mut length as *mut u32;

	let mut counter = 0;
	loop {
		counter += 1;
		if *flag.lock().unwrap() == 1 {
			break;
		}
		unsafe {
			// *const i32 を想定して書いているが *mut i32 かもしれない
			// 実機で確認
			TUSB0216AD_Ad_Data(id, 1, data1.as_mut_ptr(), l_ptr);
			TUSB0216AD_Ad_Data(id, 2, data2.as_mut_ptr(), l_ptr);
		}

		// データの変換
		// 入力範囲をもとにしてデータを電圧に変換する。
		// +/-10Vとして位置測定をしていると仮定している
		let mut tmp1 = vec![0.0; length as usize];
		let mut tmp2 = vec![0.0; length as usize];
		for i in 0..length {
			tmp1[i as usize] = (data1[i as usize] as f32) * 10.0 / 2f32.powf(16.0);
			tmp2[i as usize] = (data2[i as usize] as f32) * 10.0 / 2f32.powf(16.0);
		}
		// TODO: ここから先は非同期処理に切り出す
		// ステージのポジション(tmp1)ごとにデータをまとめる
		// +/-10Vとして位置測定をしていると仮定している
		let mut dataset = tmp1
			.iter()
			.zip(tmp2.iter())
			.map(|(x, y)| ((x * 10000.0).round() / 10000.0, y))
			.collect::<Vec<_>>();
		dataset.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
		// aggregation
		let mut data_bank: Vec<Data> = Vec::new();

		for d in dataset.iter() {
			let converted_data = Data::new(d.0, *d.1);
			match data_bank.iter().position(|data| data.x == converted_data.x) {
				Some(index) => {
					data_bank[index].y =
						data_bank[index].y * (data_bank[index].len as f32) + converted_data.y;
					data_bank[index].y /= (data_bank[index].len + 1) as f32;

					data_bank[index].len = data_bank[index].len + 1;
				}
				None => {
					data_bank.push(converted_data);
				}
			}
		}

		data_bank.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());

		// position, intensity に値を入れる
		for (idx, d) in data_bank.iter().enumerate() {
			position[idx] = d.x;
			intensity[idx] = d.y;
		}

		// データの要求長を元に戻す
		length = MAX_LENGTH as u32;
	}
	println!("{}", counter);
	println!("Data acquisition stopped");
}
