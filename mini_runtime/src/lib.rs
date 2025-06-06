use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    time::{Duration, Instant},
};
use rand::Rng;

pub struct MiniRuntime {
    ready_queue: VecDeque<Pin<Box<dyn Future<Output = ()> + 'static>>>,
    timer_queue: VecDeque<(Instant, Pin<Box<dyn Future<Output = ()> + 'static>>)>,
}

impl MiniRuntime {
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
            timer_queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, future: impl Future<Output = ()> + 'static) {
        self.ready_queue.push_back(Box::pin(future));
    }

    pub fn spawn_delayed(
        &mut self,
        delay: Duration,
        future: impl Future<Output = ()> + 'static,
    ) {
        let wake_time = Instant::now() + delay;
        self.timer_queue.push_back((wake_time, Box::pin(future)));
        self.timer_queue
            .make_contiguous()
            .sort_by(|a, b| a.0.cmp(&b.0));
    }

    pub fn run(&mut self) {
        loop {
            // 加入新spawn的任务
            SPAWN_QUEUE.with(|queue| {
                let mut queue = queue.borrow_mut();
                while let Some(fut) = queue.pop() {
                    self.spawn(fut);
                }
            });

            let mut did_work = false;

            // 处理定时任务
            let now = Instant::now();
            while let Some((wake_time, _)) = self.timer_queue.front() {
                if *wake_time <= now {
                    let (_, task) = self.timer_queue.pop_front().unwrap();
                    self.ready_queue.push_back(task);
                    did_work = true;
                } else {
                    break;
                }
            }

            // 处理就绪任务
            if let Some(mut task) = self.ready_queue.pop_front() {
                let waker = noop_waker();
                let mut cx = Context::from_waker(&waker);

                match task.as_mut().poll(&mut cx) {
                    Poll::Ready(()) => {}
                    Poll::Pending => {
                        self.ready_queue.push_back(task);
                    }
                }
                did_work = true;
            }

            // 没有就绪任务，等待下一个定时任务或退出
            if !did_work {
                if let Some((wake_time, _)) = self.timer_queue.front() {
                    let now = Instant::now();
                    if *wake_time > now {
                        std::thread::sleep(*wake_time - now);
                    }
                } else if self.ready_queue.is_empty() {
                    // 没有任何任务了
                    break;
                }
            }
        }
    }
}

fn noop_waker() -> Waker {
    unsafe fn clone(_data: *const ()) -> RawWaker { unsafe { noop_raw_waker() } }
    unsafe fn noop(_data: *const ()) {}
    const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
    unsafe fn noop_raw_waker() -> RawWaker { RawWaker::new(std::ptr::null(), &VTABLE) }
    unsafe { Waker::from_raw(noop_raw_waker()) }
}

pub async fn sleep(dur: Duration) {
    struct Sleep { wake_time: Instant }
    impl Future for Sleep {
        type Output = ();
        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            if Instant::now() >= self.wake_time {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }
    Sleep { wake_time: Instant::now() + dur }.await
}

pub async fn random_sleep(min: u64, max: u64) {
    let mut rng = rand::rng();
    let duration = Duration::from_millis(rng.random_range(min..max));
    sleep(duration).await
}

/// mini_spawn! 只把 future 放到 SPAWN_QUEUE
#[macro_export]
macro_rules! mini_spawn {
    ($($t:tt)*) => {
        $crate::SPAWN_QUEUE.with(|queue| {
            queue.borrow_mut().push(Box::pin(async { $($t)* }))
        });
    };
}

pub use spawn_queue::SPAWN_QUEUE;

mod spawn_queue {
    use std::{cell::RefCell, future::Future, pin::Pin};

    thread_local! {
        pub static SPAWN_QUEUE: RefCell<Vec<Pin<Box<dyn Future<Output = ()> + 'static>>>> =
            RefCell::new(Vec::new());
    }
}
