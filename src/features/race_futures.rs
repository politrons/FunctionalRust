use futures::{
    future::FutureExt,
    pin_mut,
    select,
};

#[cfg(test)]
mod tests {
    use std::future;
    use std::time::Duration;
    use futures::executor::block_on;
    use super::*;

    //
    // #[test]
    // fn race() {
    //     block_on(race_tasks());
    // }
    //
    // async fn race_tasks() {
    //     let mut t1 = future::ready(1);
    //     let mut t2 = future::ready(2);
    //     loop {
    //         select! {
    //             car1 = t1 => println!("${} win", car1),
    //             car2 = t2 => println!("${} win", car2),
    //             complete => break,
    //             default => unreachable!(), // never runs (futures are ready, then complete)
    //         };
    //     }
    //     // let t1 = lotus().fuse();
    //     // let t2 = ferrari().fuse();
    //
    //     // pin_mut!(t1, t2);
    //     //
    //     // select! {
    //     //     car1 = t1 => println!("${} win", car1),
    //     //     car2 = t2 => println!("${} win", car2),
    //     // }
    // }
    //
    // async fn lotus() -> String {
    //     std::thread::sleep(Duration::from_secs(2));
    //     return "Lotus".to_string();
    // }
    //
    // async fn ferrari() -> String {
    //     return "Ferrari".to_string();
    // }
}
