use tokio::sync::broadcast::Sender;
use super::stream::SseEvent;

pub struct DeltaAccumulator {
    buffer: String,
    tx: Sender<SseEvent>,
    delta_count: i32,
}

impl DeltaAccumulator {
    pub fn new(tx: Sender<SseEvent>) -> Self {
        Self {
            buffer: String::with_capacity(4096),
            tx,
            delta_count: 0,
        }
    }

    pub fn push(&mut self, content: &str, delta_type: &str, is_complete: bool) {
        if !content.is_empty() {
            self.buffer.push_str(content);
            self.delta_count += 1;
            let _ = self.tx.send(SseEvent::MessageDelta {
                content: content.to_string(),
                index: self.delta_count,
                delta_type: delta_type.to_string(),
            });
        }
        
        if is_complete {
            let _ = self.tx.send(SseEvent::MessageComplete {
                content_length: self.buffer.len(),
            });
        }
    }

    pub fn finalize(self) -> String {
        self.buffer
    }

    pub fn count(&self) -> i32 {
        self.delta_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;

    #[tokio::test]
    async fn test_delta_accumulator() {
        let (tx, mut rx) = broadcast::channel(16);
        let mut accumulator = DeltaAccumulator::new(tx);

        accumulator.push("Hello ", "markdown", false);
        accumulator.push("world!", "markdown", true);

        // Verify broadcast events
        let event1 = rx.recv().await.unwrap();
        match event1 {
            SseEvent::MessageDelta { content, index, delta_type } => {
                assert_eq!(content, "Hello ");
                assert_eq!(index, 1);
                assert_eq!(delta_type, "markdown");
            }
            _ => panic!("Expected MessageDelta"),
        }

        let event2 = rx.recv().await.unwrap();
        match event2 {
            SseEvent::MessageDelta { content, index, delta_type } => {
                assert_eq!(content, "world!");
                assert_eq!(index, 2);
                assert_eq!(delta_type, "markdown");
            }
            _ => panic!("Expected MessageDelta"),
        }

        let event3 = rx.recv().await.unwrap();
        match event3 {
            SseEvent::MessageComplete { content_length } => {
                assert_eq!(content_length, 12);
            }
            _ => panic!("Expected MessageComplete"),
        }

        let final_text = accumulator.finalize();
        assert_eq!(final_text, "Hello world!");
    }
}
