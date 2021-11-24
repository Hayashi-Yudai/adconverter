mod helpers;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use dotenv::dotenv;
use helpers::{helper, operation, post};
use std::os::raw::{c_int, c_short, c_uchar, c_uint};
use std::sync::{Arc, Mutex};
use std::thread;

#[no_mangle]
pub extern "C" fn open(id: c_short) -> i16 {
    let error: c_short;
    unsafe {
        error = operation::TUSB0216AD_Device_Open(id);
    }
    helper::parse_error(error, "TUSB0216AD_Device_Open");

    error
}

#[no_mangle]
pub extern "C" fn close(id: c_short) {
    unsafe {
        operation::TUSB0216AD_Device_Close(id);
    }
}

#[no_mangle]
pub extern "C" fn set_clock(id: c_short, clock_time: c_int, sel: c_uchar) -> c_short {
    let error: c_short;
    unsafe {
        error = operation::TUSB0216AD_AdClk_Set(id, clock_time, sel);
    }
    helper::parse_error(error, "TUSB0216AD_AdClk_Set");

    error
}

/// Change input range of each channel.
/// Specify the input ranges with a number.
/// 0: +/-10 V, 1: +/-5V, 2: +/-2.5 V, 3: +/-1.25V
/// 4: 10 V, 5: 5 V, 6: 2.5 V
///
/// * `id` - Device number
/// * `type1` - input range of CH1
/// * `type2` - input range of CH2
#[no_mangle]
pub extern "C" fn input_set(id: i32, type1: u8, type2: u8) {
    unsafe {
        let err = operation::TUSB0216AD_Input_Set(id, type1, type2);
        helper::parse_error(err, "TUSB0216AD_Input_Set");
    }
}

#[no_mangle]
pub extern "C" fn run(id: i32, seconds: u64) {
    // sequence が走っているかを示すフラグ
    // -1: not-started, 0: running, 1: finished
    let flag = Arc::new(Mutex::new(0));
    dotenv().ok();

    // +/- 3.75μm駆動させたときに精度375nmで取るために必要な領域
    const DATA_SIZE: usize = 20000;

    let flg1 = Arc::clone(&flag);
    let time_keeper = thread::spawn(move || {
        helper::continuous_read(id, seconds, flg1);
    });

    let flg2 = Arc::clone(&flag);
    let x = Arc::new(Mutex::new(Vec::<f32>::with_capacity(DATA_SIZE)));
    let y = Arc::new(Mutex::new(Vec::<f32>::with_capacity(DATA_SIZE)));
    let counter = Arc::new(Mutex::new(Vec::<u32>::with_capacity(DATA_SIZE)));

    let x_cln = Arc::clone(&x);
    let y_cln = Arc::clone(&y);
    let counter_cln = Arc::clone(&counter);
    let job_runner = thread::spawn(move || {
        helper::get_data(id, flg2, x_cln, y_cln, counter_cln);
    });

    let x_cln2 = Arc::clone(&x);
    let y_cln2 = Arc::clone(&y);
    let flg3 = Arc::clone(&flag);
    let post_data = thread::spawn(move || {
        let _ = post::post_data(flg3, x_cln2, y_cln2);
    });

    time_keeper.join().expect("Paniced at time_keeper");
    job_runner.join().expect("Paniced at job_runner");
    post_data.join().expect("Paniced at post_data thread");
}
