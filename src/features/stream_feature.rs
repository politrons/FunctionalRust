use std::time::Duration;

use futures::{pin_mut, stream, Stream, StreamExt};

/**
A `Stream` is useful when values arrive over time: websocket messages, queue
events, file lines, logs, or paginated API responses.

This example simulates an async event feed. The consumer awaits one event at a
time and stops when the remote side disconnects.
*/
fn producer_stream(app_events: Vec<AppEvent>) -> impl Stream<Item = AppEvent> {
    stream::unfold(app_events.into_iter(), |mut events| async move {
        let event = events.next()?;

        async_std::task::sleep(Duration::from_millis(1)).await;

        Some((event, events))
    })
}

async fn consumer_stream<S>(events: S) -> Vec<String>
where
    S: Stream<Item = AppEvent>,
{
    pin_mut!(events);

    let mut messages = Vec::new();

    while let Some(event) = events.next().await {
        match event {
            AppEvent::Connected(client_id) => {
                println!("Connected client: {}", client_id);
            }
            AppEvent::Message(message) => {
                messages.push(message.to_uppercase());
            }
            AppEvent::Disconnected => {
                break;
            }
        }
    }

    messages
}

#[derive(Debug, Eq, PartialEq)]
enum AppEvent {
    Connected(String),
    Message(String),
    Disconnected,
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    #[test]
    fn stream_processes_messages_until_disconnect() {
        let stream_producer = producer_stream(vec![
            AppEvent::Connected("client-1".to_string()),
            AppEvent::Message("hello".to_string()),
            AppEvent::Message("rust stream".to_string()),
            AppEvent::Disconnected,
            AppEvent::Message("ignored after disconnect".to_string()),
        ]);
        let messages = block_on(consumer_stream(stream_producer));

        assert_eq!(messages, vec!["HELLO", "RUST STREAM"]);
    }

    #[test]
    fn stream_finishes_when_no_events_exist() {
        let messages = block_on(consumer_stream(producer_stream(vec![])));

        assert!(messages.is_empty());
    }
}
