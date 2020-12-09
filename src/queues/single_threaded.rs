use alloc::boxed::Box;
use alloc::rc::Rc;
use heapless::spsc::{Consumer, Producer, Queue, SingleCore};
use heapless::ArrayLength;
/// - max 255 entries
/// - wrapper around heapless::spsc::Queue
pub struct ShortFixedSPSCQueue<T, N>
where
    N: ArrayLength<T>,
{
    queue: Queue<T, N, u8, SingleCore>,
}
pub struct ShortAllocFixedSPSCConsumer<T, N>
where
    N: ArrayLength<T>,
{
    queue: Rc<Queue<T, N, u8, SingleCore>>,
}
pub struct ShortAllocFixedSPSCProducer<T, N>
where
    N: ArrayLength<T>,
{
    queue: Rc<Queue<T, N, u8, SingleCore>>,
}

impl<T, N> ShortFixedSPSCQueue<T, N>
where
    N: ArrayLength<T>,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            queue: unsafe { Queue::u8_sc() },
        }
    }
    /// This function is unsafe because the programmer must make sure that the queue's consumer and producer endpoints are kept on a single core for their entire lifetime.
    #[inline]
    pub unsafe fn split(
        &mut self,
    ) -> (
        Producer<T, N, u8, SingleCore>,
        Consumer<T, N, u8, SingleCore>,
    ) {
        self.queue.split()
    }
}

impl<'rb, T, N> super::Producer<T> for Producer<'rb, T, N, u8, SingleCore>
where
    N: ArrayLength<T>,
{
    fn enqueue(&mut self, item: T) -> Result<(), T> {
        self.enqueue(item)
    }
}
impl<'rb, T, N> super::Consumer<T> for Consumer<'rb, T, N, u8, SingleCore>
where
    N: ArrayLength<T>,
{
    fn dequeue(&mut self) -> Option<T> {
        self.dequeue()
    }
}

impl<T, N> super::Queue<T> for ShortFixedSPSCQueue<T, N>
where
    N: ArrayLength<T>,
{
    #[inline]
    fn enqueue(&mut self, item: T) -> Result<(), T> {
        self.enqueue(item)
    }

    #[inline]
    fn dequeue(&mut self) -> Option<T> {
        self.dequeue()
    }
}
// impl<T, N> super::Consumer<T> for ShortAllocFixedSPSCConsumer<T, N>
// where
//     N: ArrayLength<T>,
// {
//     #[inline]
//     fn dequeue(&mut self) -> Option<T> {
//         self.queue.dequeue()
//     }
// }
// impl<T, N> super::Producer<T> for ShortAllocFixedSPSCProducer<T, N>
// where
//     N: ArrayLength<T>,
// {
//     #[inline]
//     fn enqueue(&mut self, item: T) -> Result<(), T> {
//         self.queue.enqueue(item)
//     }
// }
