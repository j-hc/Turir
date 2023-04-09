use std::error::Error;
use std::panic::Location;
use std::fmt;

#[derive(Debug)]
pub struct LocError<E> {
    pub err: E,
    pub loc: &'static Location<'static>,
}

impl<E> LocError<E> {
    #[track_caller]
    pub fn new(err: E) -> Self {
        Self {
            err,
            loc: Location::caller(),
        }
    }
}
impl<E: Error> Error for LocError<E> {}

impl<E: fmt::Debug> fmt::Display for LocError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{err:?}, {loc}", err = self.err, loc = self.loc)
    }
}

pub trait IntoLocErr {
    fn into_loc_err(self) -> LocError<Self>
    where
        Self: Sized;
}

impl<E: Error> IntoLocErr for E {
    #[track_caller]
    fn into_loc_err(self) -> LocError<Self> {
        LocError::new(self)
    }
}
