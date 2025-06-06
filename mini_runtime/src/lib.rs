use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    time::{Duration, Instant},
};
use rand::Rng;

/// 简易运行时
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
        while !self.ready_queue.is_empty() || !self.timer_queue.is_empty() {
            // 处理定时任务
            let now = Instant::now();
            while let Some((wake_time, _)) = self.timer_queue.front() {
                if *wake_time <= now {
                    let (_, task) = self.timer_queue.pop_front().unwrap();
                    self.ready_queue.push_back(task);
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
            } else {
                // 没有就绪任务，等待下一个定时任务
                if let Some((wake_time, _)) = self.timer_queue.front() {
                    let now = Instant::now();
                    if *wake_time > now {
                        std::thread::sleep(*wake_time - now);
                    }
                }
            }
        }
    }
}

// 创建一个什么都不做的Waker
fn noop_waker() -> Waker {
    unsafe fn clone(_data: *const ()) -> RawWaker { unsafe { noop_raw_waker() } }
    unsafe fn noop(_data: *const ()) {}
    const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
    unsafe fn noop_raw_waker() -> RawWaker { RawWaker::new(std::ptr::null(), &VTABLE) }
    unsafe { Waker::from_raw(noop_raw_waker()) }
}

/// 异步sleep
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

/// 生成随机延迟的sleep
pub async fn random_sleep(min: u64, max: u64) {
    let mut rng = rand::rng();
    let duration = Duration::from_millis(rng.random_range(min..max));
    sleep(duration).await
}

/// mini_spawn! 要求显式传入 &mut rt
#[macro_export]
macro_rules! mini_spawn {
    ($rt:expr, $($t:tt)*) => {
        $rt.spawn(async { $($t)* })
    };
}