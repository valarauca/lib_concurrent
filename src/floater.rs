
use std::cell::RefCell;
use std::sync::Arc;


///Floater is an abstraction around Arc<RefCell<T>>. It exists to modularize
///the code involved when you want to have aliased access to a memory safe
///location.
///
///The floater interfaces do not do any locking. They do not trigger the
///RefCell to track borrows. All internal methods will be inlined.
pub struct Floater<T: Sync> {
    data: Arc<RefCell<T>>
}
impl<T: Sync> Floater<T> {
    ///Build a new Floater. This simply creates the Arc<RefCell< >> wrappers.
    #[inline(always)]
    pub fn new(data: T) -> Floater<T> {
        Floater {
            data: Arc::new(RefCell::new(data))
        }
    }
    ///Get a mutable ref.
    ///
    ///This will panic if the pointer is invalid.
    ///There is no locking done at this interface, that is expected to be
    ///handled by T. The method used internally is unsafe, so there can be
    ///mutible mutable borrows existing at once. As such the interface is
    ///unsafe
    #[inline(always)]
    pub fn get_mut<'a>(&'a self) -> &'a mut T {
        unsafe{
            self.data.as_ptr().as_mut().expect("Null Pointer error!")
        }
    }

    ///Get a un-mutable ref
    ///
    ///This will panic if the pointer is invalid. There is no locking or
    ///tracking done internally. 
    #[inline(always)]
    pub fn get<'a>(&'a self) -> &'a T {
        unsafe{
            self.data.as_ptr().as_mut().expect("Null Pointer error!")
    
        }
    }
}
impl<T: Sync> Clone for Floater<T> {
    fn clone(&self) -> Floater<T> {
        Floater {
            data: self.data.clone()
        }
    }
}
