use crate::chiika_env::ChiikaEnv;
use std::task::{Poll};
use std::future::{poll_fn, Future};
use std::time::Duration;
use crate::VoidFuture;

type ChiikaCont = extern "C" fn (env: &mut ChiikaEnv, value: i64);

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn sleep(env: &'static mut ChiikaEnv, cont: ChiikaCont, n: i64) -> VoidFuture {
    async fn sleep(n: i64) {
        // Hand written part (all the rest will be macro-generated)
        tokio::time::sleep(Duration::from_secs(n as u64)).await;
    }
    let mut future = Box::pin(sleep(n));
    Box::pin(poll_fn(move |ctx| {
        match future.as_mut().poll(ctx) {
            Poll::Ready(_) => {
                cont(env, n);
                Poll::Ready(())
            }
            Poll::Pending => Poll::Pending,
        }
    }))
}

