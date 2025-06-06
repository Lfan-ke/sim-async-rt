use mini_runtime::{random_sleep, MiniRuntime, mini_spawn};
use std::time::Duration;
use mini_runtime_derive::mini_main;

#[mini_main]
async fn main() {
    println!("Starting mini runtime...");

    let mut rt = MiniRuntime::new();

    // 并发执行多个任务
    mini_spawn!(&mut rt, {
        println!("Task 1 started");
        random_sleep(100, 500).await;
        println!("Task 1 completed");
    });

    mini_spawn!(&mut rt, {
        println!("Task 2 started");
        random_sleep(200, 600).await;
        println!("Task 2 completed");
    });

    // 延迟任务
    rt.spawn_delayed(
        Duration::from_millis(800),
        async {
            println!("Delayed task started after 800ms");
            random_sleep(100, 300).await;
            println!("Delayed task completed");
        },
    );

    // 主任务也可以await
    println!("Main task waiting...");
    random_sleep(300, 700).await;
    println!("Main task completed");

    rt.run();
}