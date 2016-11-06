
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
    fn send_count(&self) -> usize {
        self.send.load(REX)
    }
    #[inline(always)]
    fn recv_count(&self) -> usize {
        self.recv.load(REX)
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

use std::marker::PhantomData;
const SEQ: Ordering = Ordering::SeqCst;

///Send Item
pub struct MRMSSender<T: Sized+'static> {
    data: Floater<ChannelCore<T>>,
    marker: PhantomData<&'static T>
}
impl<T: Sized+'static> Clone for MRMSSender<T> {
    fn clone(&self) -> MRMSSender<T> {
        self.data.get().send.fetch_add(1,SEQ);
        MRMSSender {
            data: self.data.clone(),
            marker: PhantomData
        }
    }
}
impl<T:Sized+'static> Drop for MRMSSender<T> {
    fn drop(&mut self) {
        self.data.get().send.fetch_sub(1,SEQ);
        let _ = self;
    }
}
unsafe impl<T:Sized+'static> Sync for MRMSSender<T> { }
unsafe impl<T:Sized+'static> Send for MRMSSender<T> { }
impl<T:Sized+'static> MRMSSender<T> {
    ///Sends and Item
    ///
    ///Returns Async::Ok(()) if everything happened okay
    ///Returns Async::Block(T) if the send was blocked
    ///Returns Async::Err(T) if there is no receiver to get your message
    pub fn send(&self,data: T) -> Async<(),T,T> {
        let mut ptr = self.data.get_mut();
        //failed to lock
        if ptr.poll().is_err() {
            return Async::Block(data);
        }
        //is there somebody to receive the result?
        if ptr.recv_count() == 0 {
            ptr.release();
            return Async::Err(data);
        }
        ptr.append(data);
        ptr.release();
        Async::Ok(())
    }
}

///Receiver
pub struct MRMSReceiver<T: Sized+'static> {
    data: Floater<ChannelCore<T>>,
    marker: PhantomData<&'static T>
}
impl<T: Sized+'static> Clone for MRMSReceiver<T> {
    fn clone(&self) -> MRMSReceiver<T> {
        self.data.get().recv.fetch_add(1,SEQ);
        MRMSReceiver {
            data: self.data.clone(),
            marker: PhantomData
        }
    }
}
impl<T:Sized+'static> Drop for MRMSReceiver<T> {
    fn drop(&mut self) {
        self.data.get().recv.fetch_sub(1,SEQ);
        let _ = self;
    }
}
unsafe impl<T:Sized+'static> Sync for MRMSReceiver<T> { }
unsafe impl<T:Sized+'static> Send for MRMSReceiver<T> { }
impl<T:Sized+'static> MRMSReceiver<T> {
    ///Receive items
    ///
    ///Returns Async::Ok(Option<T>) an item may have returned
    ///Returns Async::Block(()) the channel is blocked
    ///Returns Async::Err(()) if there is no sender nor messages to read
    pub fn recv(&self) -> Async<Option<T>,(),()> {
        let mut ptr = self.data.get_mut();
        //failed to lock
        if ptr.poll().is_err() {
            return Async::Block(());
        }
        //is there somebody to receive the result?
        if ptr.data.len() == 0 && ptr.send_count() == 0 {
            ptr.release();
            return Async::Err(());
        }
        let x = ptr.pop();
        ptr.release();
        Async::Ok(x)
    }
}

///Build a new MRMS Channel
///
///Accepts a sized argument to pre-size it
pub fn channel<T: Sized>(size: usize) -> (MRMSSender<T>,MRMSReceiver<T>) {
   let x = Floater::new(ChannelCore::new(size));
   let s = MRMSSender {
       data: x.clone(),
       marker: PhantomData
    };
   let r = MRMSReceiver {
       data: x.clone(),
       marker: PhantomData
    };
   (s,r)
}

#[test]
fn test_mrms_channel() {
    use std::thread;
    let (s,r) = channel::<usize>(20);
    //send values 0,1,2,3,4,5,6,7,8,9
    thread::spawn(move || {
        for x in 0..10 {
            let mut y = x;
            loop {
                match s.send(y) {
                    Async::Ok(()) => break,
                    Async::Block(z) => y = z,
                    Async::Err(_) => panic!("send exploded!")
                };
            }
        }
    });
    //recieve values
    thread::spawn( move || {
        let mut output = Vec::new();
        loop {
            match r.recv() {
                Async::Ok(z) => match z {
                    Option::None => continue,
                    Option::Some(z) => output.push(z)
                },
                Async::Block(()) => continue,
                Async::Err(()) => break
            };
        }
        assert_eq!( output[0], 0usize);
        assert_eq!( output[1], 1usize);
        assert_eq!( output[2], 2usize);
        assert_eq!( output[3], 3usize);
        assert_eq!( output[4], 4usize);
        assert_eq!( output[5], 5usize);
        assert_eq!( output[6], 6usize);
        assert_eq!( output[7], 7usize);
        assert_eq!( output[8], 8usize);
        assert_eq!( output[9], 9usize);
    });
}
