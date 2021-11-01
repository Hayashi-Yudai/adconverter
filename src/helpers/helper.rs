use crate::helpers::operation::*;
use std::sync::{Arc, Mutex, MutexGuard};
use std::{thread, time};
use synthrs::filter::{convolve, cutoff_from_frequency, lowpass_filter};

/// エラーコードからエラーの詳細を表示する
/// # Arguments
///
/// * e - エラーコード
/// * func_name - エラーの発生元のメソッド名
pub fn parse_error(e: i32, func_name: &str) {
    match e {
        0 => {}
        1 => println!("{}: Invalid ID", func_name),
        2 => println!("{}: Invalid Driver", func_name),
        3 => println!("{}: Device already opened", func_name),
        4 => println!("{}: Too many devices", func_name),
        5 => println!("{}: Failed to open device", func_name),
        6 => println!("{}: Device not found", func_name),
        8 => println!("{}: Parameters are invalid", func_name),
        9 => println!("{}: USB connection error", func_name),
        11 => println!("{}: Sequential reading", func_name),
        99 => println!("{}: Other error", func_name),
        _ => println!("{}: Other error", func_name),
    }
}

struct Data {
    x: f32,
    y: f32,
    len: u32,
}

impl Data {
    fn new(x: f32, y: f32, len: u32) -> Self {
        Data {
            x: x,
            y: y,
            len: len,
        }
    }
}

/// CH1, CH2 にセットされているレンジの番号を取得する
///
/// # Argument
///
/// * id - 装置のユニット番号選択スイッチの数字
fn get_ranges(id: i32) -> (u8, u8) {
    let mut ch1_range: u8 = 0;
    let mut ch2_range: u8 = 0;
    let err;

    unsafe {
        let ch1_range_prt = &mut ch1_range;
        let ch2_range_prt = &mut ch2_range;
        err = TUSB0216AD_Input_Check(id, ch1_range_prt, ch2_range_prt);
    }

    if err != 0 {
        parse_error(err, "TUSB0216AD_Input_Check");
    }

    (ch1_range, ch2_range)
}

/// レンジの番号からレンジ幅を計算
///
/// # Argument
///
/// * range - レンジの番号。例えば0なら +/- 10 V
fn calc_range_width(range: u8) -> f32 {
    match range {
        0 => 20.0,
        1 | 4 => 10.0,
        2 | 5 => 5.0,
        3 | 6 => 2.5,
        _ => 0.0,
    }
}

/// 装置から得られた１点のストレートバイナリを電圧に変換
///
/// # Arguments
///
/// * ch1_range - CH1のレンジ番号
/// * ch2_range - CH2のレンジ番号
/// * ch1_data - CH1から得られたストレートバイナリ形式のデータ1点
/// * ch2_data - CH2から得られたストレートバイナリ形式のデータ1点
///
/// # Returns
///
/// CH1, CH2の出力電圧値
fn convert_to_voltage(ch1_range: u8, ch2_range: u8, ch1_data: f32, ch2_data: f32) -> (f32, f32) {
    let ch1_width: f32 = calc_range_width(ch1_range);
    let ch2_width: f32 = calc_range_width(ch2_range);

    let ch1_result: f32;
    let ch2_result: f32;
    if ch1_range > 3 {
        ch1_result = ch1_data * ch1_width / 2f32.powf(16.0);
    } else {
        ch1_result = ch1_data * ch1_width / 2f32.powf(16.0) - ch1_width / 2.0;
    }

    if ch2_range > 3 {
        ch2_result = ch2_data * ch2_width / 2f32.powf(16.0);
    } else {
        ch2_result = ch2_data * ch2_width / 2f32.powf(16.0) - ch2_width / 2.0;
    }

    return (ch1_result, ch2_result);
}

/// low pass filter
/// cutoff: 30 Hz
/// sampling rate: 100 kHz
/// band: 0.01
fn lowpass(sample: Vec<f32>) -> Vec<f32> {
    let filter = lowpass_filter(cutoff_from_frequency(3000.0, 1000_000), 0.1);
    let sample: Vec<f64> = sample.into_iter().map(|x| x as f64).collect();

    convolve(&filter, sample.as_slice())
        .into_iter()
        .map(|x| x as f32)
        .collect()
}

// ステージのポジション(tmp1)ごとにデータをまとめる
// +/-10Vとして位置測定をしていると仮定している
fn update_data(
    x: Vec<f32>,
    y: Vec<f32>,
    position: &mut MutexGuard<Vec<f32>>,
    intensity: &mut MutexGuard<Vec<f32>>,
    counter: &mut MutexGuard<Vec<u32>>,
) {
    // 10 V = 3.75 μm -> 10/10000 V = 375 nm
    let dataset = x
        .iter()
        .zip(y.iter())
        .map(|(x, y)| ((x * 10000.0).round() / 10000.0, y))
        .collect::<Vec<_>>();
    // dataset.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());

    // aggregation
    let mut data_bank: Vec<Data> = Vec::new();
    for d in dataset.iter() {
        match position.iter().position(|data| *data == d.0) {
            Some(index) => {
                intensity[index] = intensity[index] * counter[index] as f32 + d.1;
                intensity[index] /= (counter[index] + 1) as f32;

                counter[index] += 1;
            }
            None => {
                position.push(d.0);
                intensity.push(*d.1);
                counter.push(1);
            }
        }
    }

    for idx in 0..position.len() {
        data_bank.push(Data::new(position[idx], intensity[idx], counter[idx]));
    }
    data_bank.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    for idx in 0..position.len() {
        position[idx] = data_bank[idx].x;
        intensity[idx] = data_bank[idx].y;
        counter[idx] = data_bank[idx].len;
    }
}

/// 装置の連続データ取り込みの制御。指定の時間だけデータ取り込みを行う
/// このメソッドではデータの取り込み開始、終了を制御するだけで装置のバッファに
/// たまったデータの取り出しは行わない
///
/// # Arguments
///
/// * id - 装置のユニット番号選択スイッチの数字
/// * seconds - データ取り込みを行う秒数
/// * flag - データ取り込み中であるかを判別するフラグ
pub fn continuous_read(id: i32, seconds: u64, flag: Arc<Mutex<i8>>) {
    println!("Timer start!");
    let sleeping_time = time::Duration::from_secs(seconds);
    let mut error: i32;

    unsafe {
        // CH1, 2ともに+/-10Vの入力を受け付ける
        // 入力が+/-10VなのはSR830の仕様
        error = TUSB0216AD_Input_Set(id, 0, 0);
        parse_error(error, "TUSB0216AD_Input_Set");
        error = TUSB0216AD_Start(id, &(2 as u8), 0, 0, 0);
        parse_error(error, "TUSB0216AD_Start");
        error = TUSB0216AD_Trigger(id);
        parse_error(error, "TUSB0216AD_Trigger");
    }

    *flag.lock().unwrap() = 0; // 計測開始のフラグを立てる
    thread::sleep(sleeping_time);

    unsafe {
        error = TUSB0216AD_Stop(id);
        parse_error(error, "TUSB0216AD_Stop");
    }

    *flag.lock().unwrap() = 1; // 計測終了のフラグを立てる
    println!("Timer stopped");
}

/// データの取り込みが行われているフラグが立っている間
/// CH1, CH2 からのデータを取得する
///
/// # Arguments
///
/// * id - 装置のユニット番号選択スイッチの数字
/// * flag - データ取り込み中であるかを判別するフラグ
/// * position - CH1のデータを収納するベクトル
/// * intensity - CH2のデータを収納するベクトル
pub fn get_data(
    id: i32,
    flag: Arc<Mutex<i8>>,
    position: Arc<Mutex<Vec<f32>>>,
    intensity: Arc<Mutex<Vec<f32>>>,
    counter: Arc<Mutex<Vec<u32>>>,
) {
    let ranges: (u8, u8) = get_ranges(id);
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

    loop {
        if *flag.lock().unwrap() == 1 {
            break;
        }
        unsafe {
            TUSB0216AD_Ad_Data(id, 0, data1.as_mut_ptr(), l_ptr);
            TUSB0216AD_Ad_Data(id, 1, data2.as_mut_ptr(), l_ptr);
        }

        // データの変換
        // data1, 2には0000 ~ FFFFの16bitsストレートバイナリが入っている
        // 設定しているレンジに応じて電圧に変換する
        let mut tmp1 = vec![0.0; length as usize];
        let mut tmp2 = vec![0.0; length as usize];
        for i in 0..length as usize {
            let result = convert_to_voltage(
                ranges.0,
                ranges.1,
                data1[i as usize] as f32,
                data2[i as usize] as f32,
            );
            tmp1[i as usize] = result.0; // position
            tmp2[i as usize] = result.1; // signal
        }

        let position_denoised = lowpass(tmp1);

        // position, intensity に値を入れる
        let mut position = position.lock().unwrap();
        let mut intensity = intensity.lock().unwrap();
        let mut counter = counter.lock().unwrap();
        update_data(
            position_denoised,
            tmp2,
            &mut position,
            &mut intensity,
            &mut counter,
        );

        // データの要求長を元に戻す
        length = MAX_LENGTH as u32;
    }
    println!("Data acquisition stopped");
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::helpers::post;
    use dotenv::dotenv;
    use nearly_eq::*;
    use rand::Rng;
    use std::f64::consts::PI;
    use std::fs::File;
    use std::io::Write;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::thread;
    use std::time::Instant;

    #[test]
    fn test_get_range() {
        let range = get_ranges(2);

        assert_eq!(range.0, 0);
        assert_eq!(range.1, 0);
    }

    /// 0: +/-10V, 1: +/-5V, 2: +/-2.5V, 3: +/-1.25V, 4: 10V, 5: 5V, 6: 2.5V
    #[test]
    fn test_calc_width() {
        let width = calc_range_width(0); // +/-10 V
        assert_eq!(width, 20.0);

        let width = calc_range_width(1); // +/-5 V
        assert_eq!(width, 10.0);

        let width = calc_range_width(2); // +/-2.5 V
        assert_eq!(width, 5.0);

        let width = calc_range_width(3); // +/-1.25 V
        assert_eq!(width, 2.5);

        let width = calc_range_width(4); // 10 V
        assert_eq!(width, 10.0);

        let width = calc_range_width(5); // 5 V
        assert_eq!(width, 5.0);

        let width = calc_range_width(6); // 2.5 V
        assert_eq!(width, 2.5);
    }

    #[test]
    fn test_converting_voltage() {
        let ch1_range = 0;
        let ch2_range = 0;
        let ch1_data = 1000.0;
        let ch2_data = 500.0;

        let result = convert_to_voltage(ch1_range, ch2_range, ch1_data, ch2_data);

        assert_eq!(result.0, ch1_data * 20.0 / 2f32.powf(16.0) - 10.0);
        assert_eq!(result.1, ch2_data * 20.0 / 2f32.powf(16.0) - 10.0);
    }

    #[test]
    fn test_lowpass() {
        const DATA_NUM: usize = 10000; // 0.1 sec
        let mut x = vec![0.0];
        let mut y = vec![0.0; DATA_NUM];
        let mut rng = rand::thread_rng();

        for i in 1..DATA_NUM {
            x.push(i as f32);
        }

        // 1 data point = 10 micro-sec
        // 1 cycle = 0.05 sec = 5000 data points
        // y = sin(2e-4 * x)
        for i in 0..DATA_NUM {
            y[i] = (x[i] * 2e-4 * 2.0 * PI as f32).sin();
        }

        let mut y_noise = y.clone();
        for i in 0..DATA_NUM {
            let noise: f32 = rng.gen();
            y_noise[i] = y[i] + 2.0 * 5e-3 * (noise - 0.5);
        }

        let denoised = lowpass(y_noise);
        println!("{}", denoised.len());

        for i in 5..(DATA_NUM - 5) {
            assert_nearly_eq!(y[i], denoised[i], 0.003);
        }

        /*
        let mut file = File::create("C:/Users/yudai/Desktop/test_lowpass.csv").unwrap();
        for i in 0..DATA_NUM {
            write!(file, "{},{}\n", y[i], denoised[i]).unwrap();
        }
        file.flush().unwrap();
        */
    }

    #[test]
    fn test_update_data() {
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![0.0, 1.0, 4.0, 9.0, 16.0];
        let position = Mutex::new(vec![0.0, 2.0, 4.0]);
        let intensity = Mutex::new(vec![0.01, 4.5, 15.8]);
        let counter = Mutex::new(vec![1, 2, 1]);

        update_data(
            x,
            y,
            &mut position.lock().unwrap(),
            &mut intensity.lock().unwrap(),
            &mut counter.lock().unwrap(),
        );

        let position = position.lock().unwrap();
        let intensity = intensity.lock().unwrap();
        let counter = counter.lock().unwrap();
        let xx = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let yy = vec![0.005, 1.0, 13.0 / 3.0, 9.0, 15.9];
        let cc = vec![2, 1, 3, 1, 2];
        for i in 0..5 {
            assert_eq!(position[i], xx[i]);
            assert_eq!(intensity[i], yy[i]);
            assert_eq!(counter[i], cc[i]);
        }
    }

    #[test]
    fn test_continuous_read() {
        let seconds = 1;
        let start = Instant::now();
        continuous_read(1, seconds, Arc::new(Mutex::new(0)));
        let end = start.elapsed();

        assert_nearly_eq!(end.as_millis() as f32, (seconds * 1000) as f32, 50.0);
    }

    #[test]
    #[ignore]
    fn test_get_data() {
        dotenv().ok();

        let position: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let intensity: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let counter: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));
        let flag = Arc::new(Mutex::new(-1));

        let flg1 = Arc::clone(&flag);
        let time_keeper = thread::spawn(move || {
            continuous_read(1, 1, flg1);
        });

        let flg2 = Arc::clone(&flag);
        let posi_cln = Arc::clone(&position);
        let intensity_cln = Arc::clone(&intensity);
        let counter_cln = Arc::clone(&counter);
        let job_runner = thread::spawn(move || {
            get_data(1, flg2, posi_cln, intensity_cln, counter_cln);
        });

        let x_cln2 = Arc::clone(&position);
        let y_cln2 = Arc::clone(&intensity);
        let flg3 = Arc::clone(&flag);
        let post_data = thread::spawn(move || {
            let _ = post::post_data(flg3, x_cln2, y_cln2);
        });

        time_keeper.join().unwrap();
        job_runner.join().unwrap();
        post_data.join().unwrap();

        assert_eq!(*Arc::clone(&flag).lock().unwrap(), 1);

        let output_posi = Arc::clone(&position);
        let output_int = Arc::clone(&intensity);
        let output_counter = Arc::clone(&counter);

        /*
        let mut file = File::create("C:/Users/yudai/Desktop/test.csv").unwrap();
        for i in 0..2588 {
            write!(
                file,
                "{},{},{}\n",
                output_posi.lock().unwrap()[i],
                output_int.lock().unwrap()[i],
                output_counter.lock().unwrap()[i]
            )
            .unwrap();
        }
        file.flush().unwrap();
        */
    }

    #[test]
    fn test_ad_data_mock() {
        const MAX_LENGTH: usize = 100000;
        let mut length = MAX_LENGTH as u32;
        let mut data1 = [0; MAX_LENGTH];
        let mut data2 = [0; MAX_LENGTH];
        let l_ptr = &mut length as *mut u32;
        let err;
        let err2;
        unsafe {
            err = TUSB0216AD_Ad_Data(1, 0, data1.as_mut_ptr(), l_ptr);
            err2 = TUSB0216AD_Ad_Data(1, 1, data2.as_mut_ptr(), l_ptr);
        }

        assert_eq!(err, 0);
        assert_eq!(err2, 0);
        assert_eq!(length, 10000);

        /*
        let mut file = File::create("C:/Users/yudai/Desktop/test_raw.csv").unwrap();
        for i in 0..9980 {
            let result = convert_to_voltage(0, 0, data1[i] as f32, data2[i] as f32);
            write!(file, "{},{}\n", result.0, result.1).unwrap();
        }
        file.flush().unwrap();
        */
    }
}
