pub mod mrms;
pub mod threadlocalkey;
pub mod floater;
pub mod spinlock;

///Async Enum
///
///A high level enum that represents the 3 states an asynchronous object
///can be in
pub enum Async<T,B,E> {
    Ok(T),
    Block(B),
    Err(E)
}
impl<T,B,E> Async<T,B,E> {
    ///returns true if the value is Ok
    #[inline(always)]
    pub fn is_ok(&self) -> bool {
        match self {
            &Async::Ok(_) => true,
            _ => false
        }
    }
    ///returns true if the value is an Error
    #[inline(always)]
    pub fn is_err(&self) -> bool {
        match self {
            &Async::Err(_) => true,
            _ => false
        }
    }
    ///returns true if the value is an Block
    #[inline(always)]
    pub fn is_blocked(&self) -> bool {
        match self {
            &Async::Block(_) => true,
            _ => false
        }
    }
    ///returns Some if value is Okay
    #[inline(always)]
    pub fn ok<'a>(&'a self) -> Option<&'a T> {
        match self {
            &Async::Ok(ref x) => Some(x),
            _ => None
        }
    }
    ///returns Some if the value is an Error
    #[inline(always)]
    pub fn err<'a>(&'a self) -> Option<&'a E> {
        match self {
            &Async::Err(ref x) => Some(x),
            _ => None
        }
    }
    ///returns Some if the value is Blocked
    #[inline(always)]
    pub fn block<'a>(&'a self) -> Option<&'a B> {
        match self {
            &Async::Block(ref x) => Some(x),
            _ => None
        }
    }
    ///converts values to references of themsevles
    #[inline(always)]
    pub fn as_ref<'a>(&'a self) -> Async<&'a T, &'a B, &'a E> {
        match self {
            &Async::Ok(ref x) => Async::Ok(x),
            &Async::Err(ref x) => Async::Err(x),
            &Async::Block(ref x) => Async::Block(x)
        }
    }
}
impl<T:PartialEq,B:PartialEq,E:PartialEq> PartialEq for Async<T,B,E> {
    fn eq(&self,other: &Async<T,B,E>) ->bool {
        match self {
            &Async::Ok(ref x) => match other {
                &Async::Ok(ref y) => x == y,
                _ => false
            },
            &Async::Err(ref x) => match other {
                &Async::Err(ref y) => x == y,
                _ => false
            },
            &Async::Block(ref x) => match other {
                &Async::Block(ref y) => x == y,
                _ => false
            }
        }
    }
}
impl<T:Eq,B:Eq,E:Eq> Eq for Async<T,B,E> { }

