use tokio::fs;

#[no_mangle]
pub extern "C" fn print(n: i64) -> i64 {
    println!("{}", n);
    n
}

async fn read(_: i64) -> i64 {
    match fs::read_to_string("count.txt").await {
        Ok(s) => s.parse().unwrap(),
        Err(_) => 0,
    }
}

async fn write(n: i64) -> i64 {
    let _ = fs::write("count.txt", n.to_string()).await;
    0
}

extern "C" {
    fn chiika_main(_: i64) -> i64;
}

#[no_mangle]
pub extern "C" fn chiika_start_tokio(_: i64) -> i64 {
     tokio::runtime::Builder::new_multi_thread()
         .enable_all()
         .build()
         .unwrap()
         .block_on(async {
             unsafe { chiika_main(0) }
         })

    // Q: Need this?
    // sleep(Duration::from_millis(50)).await;
}
