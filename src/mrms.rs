
use super::Async;
use super::spinlock::{LoanLock,Lock};
use super::floater::Floater;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize,Ordering};

const REX: Ordering = Ordering::SeqCst;


struct ChannelCore<T: Sized> {
    send: AtomicUsize,
    recv: AtomicUsize,
    lock: AtomicUsize,
    data: VecDeque<T>
}
impl<T: Sized> ChannelCore<T> {
    fn new(size: usize) -> ChannelCore<T> {
        ChannelCore {
            send: AtomicUsize::new(1),
            recv: AtomicUsize::new(1),
            lock: AtomicUsize::new(0),
            data: VecDeque::<T>::with_capacity(size)
        }
    }
    #[inline(always)]
    fn size(&self) -> usize {
        self.data.len()
    }
    #[inline(always)]
    fn send_count(&self) -> usize {
        self.send.load(REX)
    }
    #[inline(always)]
    fn recv_count(&self) -> usize {
        self.recv.load(REX)
    }
    #[inline(always)]
    fn capacity(&self) -> usize {
        self.data.capacity()
    }
    #[inline(always)]
    fn append(&mut self, data: T) {
        self.data.push_back(data);
    }
    #[inline(always)]
    fn pop(&mut self) -> Option<T> {
        self.data.pop_front()
    }
}
unsafe impl<T:Sized> Sync for ChannelCore<T> { }
impl<T: Sized> LoanLock for ChannelCore<T> {
    fn loan<'a>(&'a self) -> &'a AtomicUsize {
        &self.lock
    }
}

#[derive(Clone)]
struct MSMRChannel<T:Sized> {
    data: Floater<ChannelCore<T>>
}

pub struct MRMSSender<T: Sized> {
    data: MSMRChannel<T>
}

