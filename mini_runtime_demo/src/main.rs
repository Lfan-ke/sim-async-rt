use mini_runtime::{SPAWN_QUEUE, mini_chain, mini_gather, mini_spawn, random_sleep, sleep};
use mini_runtime_derive::mini_main;
use std::time::Duration;

async fn rd_sleep(min: u64, max: u64) {
    println!("Task 3 started");
    random_sleep(min, max).await;
    println!("Task 3 completed");
}

async fn od_sleep(min: u64, max: u64, index: usize) {
    println!("Task [od] {index} started");
    random_sleep(min, max).await;
    println!("Task [od] {index} completed");
}

async fn gp_sleep(min: u64, max: u64) {
    println!("Task gp[{}, {}] started", min, max);
    random_sleep(min, max).await;
    println!("Task gp[{}, {}] completed", min, max);
}

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

    mini_chain!(
        od_sleep(0, 700, 0),
        od_sleep(0, 1000, 1),
        od_sleep(0, 1000, 2)
    );

    mini_gather![gp_sleep(0, 700), gp_sleep(0, 1000)];

    mini_spawn! {
        async {
            println!("Task 4 started");
            random_sleep(0, 2000).await;
            println!("Task 4 completed");
        }.await
    }

    mini_spawn! {rd_sleep(0, 1000).await}

    println!("Main task waiting...");
    random_sleep(300, 700).await;
    println!("Main task completed");
}
