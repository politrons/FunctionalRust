use std::thread;
use std::time::Duration;
use futures::channel::oneshot;
use futures::{pin_mut, FutureExt};
use rand::Rng;

/**
In tokio we can use [spawn] to run any async process i a [green thread].
In case we want to run several task in parallel and make a [race] like in [ZIO] or other systems,
we can use [select!] which it will be subscribe to all futures in progress, and once it detects that
one finish, it will cancel the rest of the futures in progress.

[select!] it work with [tokio] [fuse], so it subscribe to all [Fuse] each of them associated with
as callback operation. And once the [fuse] respond the data, it invoke the callback

Here we emulate a Race between cars where once [select!] detect the first car in finish the race,
we automatically invoke the callback associated, and it cancel the rest of futures.
*/
pub async fn race_condition() {

    let rnd = rand::thread_rng().gen_range(0..100);
   let porche = tokio::spawn(async move {
        let car = "Porsche";
        thread::sleep(Duration::from_millis(rnd));
        println!("{} running race in track {:?}",car, thread::current().id());
       "Porche"
    }).fuse();
    let ferrari = tokio::spawn(async move {
        let car = "Ferrari";
        thread::sleep(Duration::from_millis(rnd));
        println!("{} running race in track {:?}",car, thread::current().id());
        "Ferrari"
    }).fuse();
    let lotus = tokio::spawn(async move {
        let car = "Lotus";
        thread::sleep(Duration::from_millis(rnd));
        println!("{} running race in track {:?}",car, thread::current().id());
        "Lotus"
    }).fuse();

    pin_mut!(porche, ferrari, lotus);

    tokio::select! {
        winner = porche => {
            println!("{:?} win the race ", winner.unwrap());
        }
        winner = ferrari => {
            println!("{:?} win the race ", winner.unwrap());
        }
         winner = lotus => {
            println!("{:?} win the race ", winner.unwrap());
        }
    }
}

mod test{
    #[tokio::main]
    #[test]
    async fn main() {
        super::race_condition().await;
    }
}