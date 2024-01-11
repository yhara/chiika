#[repr(C)]
#[derive(Debug)]
pub struct ChiikaEnv {
    stack: Vec<i64>,
}

impl ChiikaEnv {
    pub fn new() -> ChiikaEnv {
        ChiikaEnv {
            stack: vec![],
        }
    }
}

#[no_mangle]
pub extern "C" fn chiika_env_push(env: *mut ChiikaEnv, item: i64) {
    unsafe { (*env).stack.push(item); }
}

#[no_mangle]
pub extern "C" fn chiika_env_pop(env: *mut ChiikaEnv) -> i64 {
    unsafe { (*env).stack.pop().unwrap() }
}
