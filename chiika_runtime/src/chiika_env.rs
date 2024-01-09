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
