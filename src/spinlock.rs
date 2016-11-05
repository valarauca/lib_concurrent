//!Spin lock abstract traits.
//!
//!The goal of this crate it to allow for the abstraction of spin locking. 
//!Effectively. This means a resource which requires locking behavior can
//!implement one of these traits `CanLock`. By doing this it will gain a
//!huge number of additional functionality. 
//!
//!The cascading/inherated traits from `CanLock` are for `AtomicUsize`.
//!
//!      use self::lib_concurrent::spinlock::{LoanLock,Convar,IntoConvar};
//!      use std::collections::VecDeque;
//!      use std::sync::atomic::AtomicUsize;
//!      
//!      //build a structure that requires synchronization
//!      struct CrappyQueue {
//!         data: VecDeque<u64>,
//!         lock: AtomicUsize
//!      }
//!      unsafe impl Sync for CrappyQueue{ }
//!
//!      //allow it to loan it's lock out
//!      impl LoanLock for CrappyQueue {
//!         fn loan<'a>(&'a self) -> &'a AtomicUsize {
//!             &self.lock
//!         }
//!      }
//!
//!      //Build item
//!      let mut x = CrappyQueue {
//!         data: VecDeque::new(),
//!         lock: AtomicUsize::new(0)
//!      };
//!      //get test data
//!      x.data.push_front(2);
//!      x.data.push_front(3);
//!      assert!(x.data.len() == 2);
//!      //build the convar
//!      let con = x.into_convar();
//!      //we don't own the convar yet
//!      assert!( ! con.is_locked() );
//!      //we can't get the data since we don't have a lock
//!      assert!( con.data().is_none() );
//!      //lock it, I'm the only user so it'll lock
//!      con.poll();
//!      //it should be locked
//!      assert!( con.is_locked() );
//!      //and we can see the data
//!      assert!( con.data().unwrap().data.len() == 2);

use std::time::{Duration,Instant};
use std::sync::atomic::{AtomicBool,AtomicUsize,Ordering};
const SEQ: Ordering = Ordering::SeqCst;
const REX: Ordering = Ordering::Relaxed;

///Represents the various states a lock can be in.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum LockState {
    Ready,
    Block,
    Timeout
}

///Represents the abstraction notion that a resource must be acquired or
///locked. Contained is the act of polling the lock, and releasing the lock.
pub trait Lock {
    fn poll(&self) -> LockState;
    fn release(&self);
}


///Trait for when a larger type wants to build up a lock. This loans an
///internal atomic 
pub trait LoanLock {
    fn loan<'a>(&'a self) -> &'a AtomicUsize;
}

///A type that is turning into a lock.
pub trait IntoLock<'a> {
    type Item: Lock+'a;
    fn into_lock(&'a self) -> Self::Item;
    fn into_timeout(&'a self, period: Duration) -> Self::Item;
}

///Abstract type that encapsulates a lock. 
pub struct AtomicLocker<'a> {
    data: &'a AtomicUsize,
    start: Option<Instant>,
    period: Option<Duration>
}
impl<'a> Lock for AtomicLocker<'a> {
    fn poll(&self) -> LockState {
        //check if we have a lock
        if self.data.compare_and_swap(0,1,SEQ)==0 {
            return LockState::Ready;
        }
        //check timeout
        match self.period {
            Option::None => LockState::Block,
            Option::Some(d) => match self.start {
                Option::None => unreachable!(),
                Option::Some(s) => if s.elapsed() >= d {
                    LockState::Timeout
                } else {
                    LockState::Block
                }
            }
        }
    }
    fn release(&self) {
        self.data.store(0,REX);
    }
}
impl<'a, T: LoanLock> IntoLock<'a> for T {
    type Item = AtomicLocker<'a>;
    fn into_lock(&'a self) -> AtomicLocker<'a> {
        AtomicLocker {
            data: self.loan(),
            start: None,
            period: None
        }
    }
    fn into_timeout(&'a self, period: Duration) -> AtomicLocker<'a> {
        AtomicLocker{
            data: self.loan(),
            start: Some(Instant::now()),
            period: Some(period)
        }
    }
}

///An Abstract Convar.
///
///Represents a structure attempting to become locked, or is
///currently locked. When this structure passes out of scope it will
///automagically unlock, if it achieved a lock.
///
pub struct Convar<'a,T: Sync+Sized+LoanLock+'a> {
    data: &'a T,
    flag: AtomicBool,
    lock: AtomicLocker<'a>
}
impl<'a,T: Sync+Sized+LoanLock+'a> Convar<'a,T> {
    
    ///Does this instance hold the lock?
    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        self.flag.load(REX)
    }

    ///Attempt to acquire the lock.
    pub fn poll(&self) -> LockState {
        if self.is_locked() {
            return LockState::Ready;
        }
        let x = self.lock.poll();
        if x == LockState::Ready {
            self.flag.store(true,REX);
        }
        x
    }

    ///Return pointer to underlying data. This only happens when
    ///the structure is locked. It returns NONE otherwise
    pub fn data(&self) -> Option<&'a T> {
        if self.is_locked() {
            Some(self.data)
        } else {
            None
        }
    }
}
impl<'a,T: Sync+Sized+LoanLock+'a> Drop for Convar<'a,T> {
    fn drop(&mut self) {
        if self.is_locked() {
            self.lock.release();
        }
        let _ = self;
    }
}

pub trait IntoConvar:Sync+Sized+LoanLock {
    fn into_convar<'a>(&'a self) -> Convar<'a,Self>;
    fn timeout_convar<'a>(&'a self, period: Duration) -> Convar<'a,Self>;
}
impl<T:Sync+Sized+LoanLock+'static> IntoConvar for T {
    fn into_convar<'a>(&'a self) -> Convar<'a,T> {
        Convar {
            data: self,
            flag: AtomicBool::new(false),
            lock: self.into_lock()
        }
    }
    fn timeout_convar<'a>(&'a self, period: Duration) -> Convar<'a,T> {
        Convar {
            data: self,
            flag: AtomicBool::new(false),
            lock: self.into_timeout(period)
        }
    }
}
