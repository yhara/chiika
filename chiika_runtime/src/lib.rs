#[no_mangle]
pub extern "C" fn print(n: i64) -> i64 {
    println!("{}", n);
    n
}
