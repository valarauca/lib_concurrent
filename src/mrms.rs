
//use super::spinlock::{LoanLock,Convar,IntoConvar};
//use std::collects::VecDeque;
//use std::sync::atomic::AtomicUsize;

///The Abstract contract a channel has to be able to commit too.
///
///Generalized for SRSS (Single Receiver Single Sender). The trait
///abstraction is created so the concrete channels can have their
///entire guts replaced without too much drama.
pub trait ChannelCore<T: Sized+Send> {
    fn has_send(&self) -> bool;
    fn has_recv(&self) -> bool;
    fn size(&self) -> usize;
    fn send(&self, data: T) -> Result<(),T>;
    fn receive(&self) -> Result<T,()>;
}
