use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::types::ReviewComment;

#[derive(Debug)]
pub struct CommentQueue {
    queue: VecDeque<ReviewComment>,
    next_id: AtomicU64,
}

impl CommentQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            next_id: AtomicU64::new(1),
        }
    }

    pub fn enqueue(
        &mut self,
        file: String,
        start_line: u32,
        end_line: u32,
        code_context: String,
        comment: String,
    ) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        self.queue.push_back(ReviewComment {
            id,
            file,
            start_line,
            end_line,
            code_context,
            comment,
        });
        id
    }

    pub fn dequeue(&mut self) -> Option<ReviewComment> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

pub struct AppState {
    pub comment_queue: Arc<Mutex<CommentQueue>>,
    pub repo_path: Arc<Mutex<Option<String>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            comment_queue: Arc::new(Mutex::new(CommentQueue::new())),
            repo_path: Arc::new(Mutex::new(None)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enqueue_dequeue() {
        let mut queue = CommentQueue::new();
        let id = queue.enqueue(
            "src/main.rs".into(),
            10,
            12,
            "let x = 1;".into(),
            "Use a constant".into(),
        );
        assert_eq!(id, 1);
        assert_eq!(queue.len(), 1);

        let comment = queue.dequeue().unwrap();
        assert_eq!(comment.id, 1);
        assert_eq!(comment.file, "src/main.rs");
        assert_eq!(comment.start_line, 10);
        assert_eq!(comment.end_line, 12);
        assert_eq!(comment.comment, "Use a constant");
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_fifo_order() {
        let mut queue = CommentQueue::new();
        queue.enqueue("a.rs".into(), 1, 1, "".into(), "first".into());
        queue.enqueue("b.rs".into(), 2, 2, "".into(), "second".into());

        assert_eq!(queue.dequeue().unwrap().comment, "first");
        assert_eq!(queue.dequeue().unwrap().comment, "second");
        assert!(queue.dequeue().is_none());
    }

    #[test]
    fn test_dequeue_empty() {
        let mut queue = CommentQueue::new();
        assert!(queue.dequeue().is_none());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_ids_increment() {
        let mut queue = CommentQueue::new();
        let id1 = queue.enqueue("a.rs".into(), 1, 1, "".into(), "a".into());
        let id2 = queue.enqueue("b.rs".into(), 1, 1, "".into(), "b".into());
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }
}
