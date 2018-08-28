pub trait RingBuffer<T> {
    fn push(&mut self, t: T) -> Option<T>;
    fn pull(&mut self) -> Option<T>;
}

#[macro_use]
extern crate static_assertions;

#[macro_export]
pub use self::static_assertions::const_assert;
use std::fmt;

pub fn power_of_2(i: usize) -> bool {
    i & (i - 1) == 0
}
//$crate::const_assert!(concat!(stringify!($path), $stringify!($N)), $crate::power_of_2($N) );

#[macro_export]
macro_rules! impl_ring_buffer {

    ($( $N:expr ),+ ) => {
        use std::iter::{IntoIterator,Iterator};


        #[derive(Clone)]
        pub struct RingBuffer<S> {
            store: S,
            write: usize,
            read: usize,
            len: usize,
        }

        pub fn new<S,I>(store: S) -> RingBuffer<S> where RingBuffer<S>: $crate::RingBuffer<I> {
            RingBuffer {
                store,
                write: 0,
                read: 0,
                len: 0,
            }
         }

         impl<S> ::std::fmt::Debug for RingBuffer<S> {
            fn fmt(&self, f: &'_ mut ::std::fmt::Formatter<'_>) -> Result<(), ::std::fmt::Error> {
                write!(f, "read: {}, write: {}", self.read, self.write)
            }
         }

        pub struct RingBufferIter<S> {
            n: usize,
            offset: usize,
            len: usize,
            buf: S
        }

          $(
          impl<I> $crate::RingBuffer<I> for RingBuffer<[Option<I>;$N]> {
             fn push(&mut self, t: I) -> Option<I> {
                 if self.len == $N {
                    if self.write & ($N - 1) == self.read & ($N - 1) {
                        self.read = self.read.wrapping_add(1);
                    }
                 } else {
                    self.len += 1;
                 }
                 self.write = self.write.wrapping_add(1);
                 ::std::mem::replace(&mut self.store[self.write & ($N- 1)], Some(t))
             }

             fn pull(&mut self) -> Option<I> {
                if self.len > 0 {
                 if self.read != self.write {
                     self.read = self.read.wrapping_add(1);
                 }
                 self.len -= 1;
                 ::std::mem::replace(&mut self.store[self.read & ($N - 1)], None)
                } else { None }

             }
          }


           impl<I> ::std::iter::Iterator for RingBufferIter<[Option<I>; $N]> {
                type Item = I;
                fn next(&mut self) -> Option<Self::Item> {
                    if self.n == self.len {
                        None
                    } else {
                    let result = self.buf[(self.n + self.offset) % $N].take();
                    self.n += 1;
                    result
                    }
                }
           }

           impl<'a, I> ::std::iter::Iterator for RingBufferIter<&'a [Option<I>; $N]> {
                type Item = &'a I;
                fn next(&mut self) -> Option<Self::Item> {
                    if self.n == self.len {
                        None
                    } else {
                    let result = self.buf[(self.n + self.offset) % $N].as_ref();
                    self.n += 1;
                    result
                    }
                }
           }

           impl<I> ::std::iter::IntoIterator for RingBuffer<[Option<I>;$N]> {
                type Item=I;
                type IntoIter = RingBufferIter<[Option<I>;$N]>;
                fn into_iter(self) -> Self::IntoIter {
                    RingBufferIter {
                        n: 0,
                        buf: self.store,
                        offset: (self.read & ($N -1)) + 1,
                        len: self.len,
                    }
              }
          }

          impl<I> RingBuffer<[Option<I>;$N]> {
              pub fn iter(&self) -> RingBufferIter<&'_ [Option<I>;$N]> {
                RingBufferIter {
                    n: 0,
                    buf: &self.store,
                    offset: (self.read & ($N -1)) + 1,
                    len: self.len
                }
              }
          }
        )*
      }
}

#[macro_export]
macro_rules! ring_buffer {
    ($P:ident, $N:expr) => {{
        let buf = [None; $N];
        $P {
            store: buf,
            write: 0,
            read: 0,
            len: 0,
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    mod ring_buffers {
        impl_ring_buffer!(4);
    }
    #[test]
    fn it_works() {
        let buf = [1,2,3];
        let mut rb = ring_buffers::new([None; 4]);
        assert_eq!(rb.pull(), None);
        assert_eq!(rb.push("1"), None);
        assert_eq!(rb.push("2"), None);
        assert_eq!(rb.push("3"), None);
        assert_eq!(rb.push("4"), None);
        assert_eq!(rb.push("5"), Some("1"));
        assert_eq!(rb.pull(), Some("2"));

        assert_eq!(rb.pull(), Some("3"));

        assert_eq!(rb.push("6"), None);
        assert_eq!(rb.push("7"), None);
        assert_eq!(rb.push("8"), Some("4"));

        println!("{:?}", rb);
        assert_eq!(rb.clone().into_iter().collect::<Vec<&str>>(), vec!["5","6","7", "8"]);
        assert_eq!(rb.iter().collect::<Vec<&&str>>(), vec![&"5",&"6",&"7",&"8"]);
        assert_eq!(rb.pull(), Some("5"));
        assert_eq!(rb.pull(), Some("6"));
        assert_eq!(rb.pull(), Some("7"));
        assert_eq!(rb.pull(), Some("8"));
        assert_eq!(rb.pull(), None);
        assert_eq!(rb.push("9"), None);
        assert_eq!(rb.pull(), Some("9"));

        rb.push("1");
        rb.push("2");
        rb.push("3");

        assert_eq!(rb.clone().into_iter().collect::<Vec<&str>>(), vec!["1","2","3"]);
        assert_eq!(rb.iter().collect::<Vec<&&str>>(), vec![&"1",&"2",&"3"]);
        assert_eq!(rb.pull(), Some("1"));
        assert_eq!(rb.clone().into_iter().collect::<Vec<&str>>(), vec!["2", "3"]);
        assert_eq!(rb.iter().collect::<Vec<&&str>>(), vec![&"2",&"3"]);
    }
}
