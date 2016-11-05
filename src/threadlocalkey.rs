

use std::thread::LocalKey;
use std::cell::RefCell;


///Method of mutating ThreadLocalKeys. It expects the local key to hold a
///RefCell. Ttherefore it handles getting a mutable pointer to the thread
///key.
///
///This will not trigger a borrow of the RefCell. So if you are doing
///co-routines you can have multiple mutable pointers at once.
///
pub fn with_mut<T,F,R>(key: &'static LocalKey<RefCell<T>>, lambda: F) -> R
where
    T: 'static,
    R: 'static,
    F: FnOnce(&mut T) -> R,
{
    key.with(|cell| {
        let mut ptr: &mut T = unsafe{cell.as_ptr().as_mut().unwrap()};
        lambda(ptr)
    })
}
