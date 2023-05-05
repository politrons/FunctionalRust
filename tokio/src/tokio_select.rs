use std::thread;
use futures::channel::oneshot;

/**
In tokio we can use [spawn] to run any async process i a [green thread].
In case we want to run several task in parallel and make a [race] like in [ZIO] or other systems,
we can use [select!] which it will be subscribe to all futures in progress, and once it detects that
one finish, it will cancel the rest of the futures in progress.

[select!] it work with [tokio] [channels], so it subscribe to all [receivers] each of them associated with
as callback operation. And once the received [receive] the data, it invoke the callback

Here we emulate a Race between cars where once [select!] detect the first car in finish the race,
we automatically invoke the callback associated, and it cancel the rest of futures.
*/
pub async fn race_condition() {
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();
    let (tx3, rx3) = oneshot::channel();

    tokio::spawn(async {
        let car = "Porsche";
        println!("{} running race in track {:?}",car, thread::current().id());
        let _ = tx1.send(car);
    });
    tokio::spawn(async {
        let car = "Ferrari";
        println!("{} running race in track {:?}",car, thread::current().id());
        let _ = tx2.send(car);
    });
    tokio::spawn(async {
        let car = "Lotus";
        println!("{} running race in track {:?}",car, thread::current().id());
        let _ = tx3.send(car);
    });

    tokio::select! {
        winner = rx1 => {
            println!("{:?} win the race ", winner);
        }
        winner = rx2 => {
            println!("{:?} win the race ", winner);
        }
         winner = rx3 => {
            println!("{:?} win the race ", winner);
        }
    }
}