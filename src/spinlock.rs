//!Spin lock abstract traits.


use std::sync::atomic::{AtomicUsize,Ordering};
const SEQ: Ordering = Ordering::SeqCst;

///Trait for when a larger type wants to build up a lock. This loans an
///internal atomic 
pub trait LoanLock {
    fn loan<'a>(&'a self) -> &'a AtomicUsize;
}

///Represents the state of a lock. Poll returns an OK(()) on lock success,
///and Err(()) is that the attempt to lock failed
pub trait Lock {
    fn poll(&self) -> Result<(),()>;
    fn release(&self);
}
impl<L: LoanLock> Lock for L {
    fn poll(&self) -> Result<(),()>{
        if self.loan().compare_and_swap(0,1,SEQ) == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
    fn release(&self) {
        self.loan().store(0,SEQ);
    }
}
