/// エラーコードからエラーの詳細を表示する
/// # Arguments
///
/// * e - エラーコード
/// * func_name - エラーの発生元のメソッド名
pub fn parse_error(e: i16, func_name: &str) {
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
