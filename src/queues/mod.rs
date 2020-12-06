pub mod single_threaded;

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
