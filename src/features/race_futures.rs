use futures::{
    future::FutureExt,
    pin_mut,
    select,
};

#[cfg(test)]
mod tests {
    use std::{future, thread};
    use std::time::Duration;

    use futures::executor::block_on;
    use rand::Rng;
    use super::*;


    #[test]
    fn race() {
        block_on(race_tasks());
    }

    async fn race_tasks() {
        let lotus = async_std::task::spawn(async  {
            let delay = rand::thread_rng().gen_range(0..100);
            thread::sleep(Duration::from_millis(delay));
            return "Lotus"
        }).fuse();
        let ferrari =  async_std::task::spawn(async  {
            let delay = rand::thread_rng().gen_range(0..100);
            thread::sleep(Duration::from_millis(delay));
            "Ferrari"
        }).fuse();

        pin_mut!(lotus, ferrari);

        select! {
            car1 = lotus => println!("{} win", car1),
            car2 = ferrari => println!("{} win", car2),
        }
    }

}
