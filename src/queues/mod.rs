pub mod single_threaded;

// https://en.wikipedia.org/wiki/Software_design_pattern#Concurrency_patterns
pub trait Queue<T> {
    fn enqueue(&mut self, item: T) -> Result<(), T>;
    fn dequeue(&mut self) -> Option<T>;
}

pub trait Consumer<T> {
    fn dequeue(&mut self) -> Option<T>;
}
pub trait Producer<T> {
    fn enqueue(&mut self, item: T) -> Result<(), T>;
}
