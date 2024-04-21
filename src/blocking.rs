// <https://doc.rust-lang.org/nightly/std/task/trait.Wake.html#examples>.
use std::*;

struct ThreadWaker(thread::Thread);

impl task::Wake for ThreadWaker {
    fn wake(self: sync::Arc<Self>) {
        self.0.unpark();
    }
}

pub fn block_on<T>(fut: impl future::Future<Output = T>) -> T {
    let mut fut = pin::pin!(fut);
    let waker = sync::Arc::new(ThreadWaker(thread::current())).into();
    let mut cx = task::Context::from_waker(&waker);
    loop {
        match fut.as_mut().poll(&mut cx) {
            task::Poll::Ready(res) => return res,
            task::Poll::Pending => thread::park(),
        }
    }
}
