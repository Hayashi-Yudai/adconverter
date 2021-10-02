mod helper;

use std::sync::{Arc, Mutex};
use std::thread;

#[no_mangle]
pub extern "C" fn open(id: i32) -> i32 {
    let error: i32;
    unsafe {
        error = helper::TUSB0216AD_Device_Open(id);
    }
    helper::parse_error(error, "TUSB0216AD_Device_Open");

    error
}

#[no_mangle]
pub extern "C" fn close(id: i32) {
    unsafe {
        helper::TUSB0216AD_Device_Close(id);
    }
}

// fn TUSB0216AD_AdClk_Set(id: i32, ClkTime: i32, sel: u8) -> i32;
#[no_mangle]
pub extern "C" fn set_clock(id: i32, clock_time: i32, sel: u8) -> i32 {
    let error: i32;
    unsafe {
        error = helper::TUSB0216AD_AdClk_Set(id, clock_time, sel);
    }
    helper::parse_error(error, "TUSB0216AD_AdClk_Set");

    error
}

#[no_mangle]
pub extern "C" fn run(id: i32, seconds: u64) {
    // sequence が走っているかを示すフラグ
    // -1: not-started, 0: running, 1: finished
    let flag = Arc::new(Mutex::new(0));
    const DATA_SIZE: usize = 50000;

    let flg1 = Arc::clone(&flag);
    let time_keeper = thread::spawn(move || {
        helper::continuous_read(id, seconds, flg1);
    });

    let flg2 = Arc::clone(&flag);
    let x = Arc::new(Mutex::new(vec![0.0 as f32; DATA_SIZE]));
    let y = Arc::new(Mutex::new(vec![0.0 as f32; DATA_SIZE]));

    let x_cln = Arc::clone(&x);
    let y_cln = Arc::clone(&y);
    let job_runner = thread::spawn(move || {
        helper::get_data(id, flg2, x_cln, y_cln);
    });

    // TODO: reqwest package を使って Django側にデータを投げる
    // TODO: データを投げる間隔は 500 msくらいにする

    time_keeper.join().unwrap();
    job_runner.join().unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    // test open() returns device not found error
    #[test]
    fn test_open_device() {
        assert_eq!(open(1), 0);
    }

    #[test]
    fn test_set_clock() {
        let mut clock_time = 1000;
        let sel = 0;

        // device not found error
        assert_eq!(set_clock(1, clock_time, sel), 6);

        clock_time = 0;
        // error with smaller number take precedence over that with larger number
        assert_eq!(set_clock(1, clock_time, sel), 6);
    }
}
