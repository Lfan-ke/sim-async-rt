use mini_runtime::{SPAWN_QUEUE, mini_spawn, random_sleep, sleep};
use mini_runtime_derive::mini_main;
use std::time::Duration;

#[mini_main]
async fn main() {
    println!("Starting mini runtime...");

    mini_spawn! {
        println!("Task 1 started");
        random_sleep(100, 500).await;
        println!("Task 1 completed");
    }

    mini_spawn! {
        println!("Task 2 started");
        random_sleep(200, 600).await;
        println!("Task 2 completed");
    }

    SPAWN_QUEUE.with(|queue| {
        queue.borrow_mut().push(Box::pin(async {
            sleep(Duration::from_millis(800)).await;
            println!("Delayed task started after 800ms");
            random_sleep(100, 300).await;
            println!("Delayed task completed");
        }));
    });

    println!("Main task waiting...");
    random_sleep(300, 700).await;
    println!("Main task completed");
}
