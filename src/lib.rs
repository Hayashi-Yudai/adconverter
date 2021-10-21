mod helpers;

use dotenv::dotenv;
use helpers::{helper, operation, post};
use std::sync::{Arc, Mutex};
use std::thread;

#[no_mangle]
pub extern "C" fn open(id: i32) -> i32 {
    let error: i32;
    unsafe {
        error = operation::TUSB0216AD_Device_Open(id);
    }
    helper::parse_error(error, "TUSB0216AD_Device_Open");

    error
}

#[no_mangle]
pub extern "C" fn close(id: i32) {
    unsafe {
        operation::TUSB0216AD_Device_Close(id);
    }
}

// fn TUSB0216AD_AdClk_Set(id: i32, ClkTime: i32, sel: u8) -> i32;
#[no_mangle]
pub extern "C" fn set_clock(id: i32, clock_time: i32, sel: u8) -> i32 {
    let error: i32;
    unsafe {
        error = operation::TUSB0216AD_AdClk_Set(id, clock_time, sel);
    }
    helper::parse_error(error, "TUSB0216AD_AdClk_Set");

    error
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

    time_keeper.join().unwrap();
    job_runner.join().unwrap();
    post_data.join().unwrap();
}
