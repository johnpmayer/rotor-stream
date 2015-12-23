use std::any::Any;
use rotor::Scope;

use {Transport, Request, StreamSocket};


// #[derive(Clone, Clone)]
// This could be Copy, but I think it could be implemented efficient enough
// without Copy and Clone. Probably we will enable them for the user code later
#[derive(Debug)]
pub enum Expectation {
    /// Read number of bytes
    ///
    /// The buffer that is passed to bytes_read might contain more bytes, but
    /// `num` will contain a number of bytes passed into `Bytes` constructor.
    ///
    /// Note that real number of bytes that `netbuf::Buf` might contain is less
    /// than 4Gb. So this value can't be as big as `usize::MAX`
    Bytes(usize),
    /// Read until delimiter, but no more than N bytes.
    ///
    /// Only static strings are support for delimiter now.
    ///
    /// `bytes_read` action gets passed `num` bytes before the delimeter, or
    /// in other words, the position of the delimiter in the buffer.
    /// The delimiter is guaranteed to be in the buffer too.
    Delimiter(&'static [u8], usize),
    /// Wait until no more than N bytes is in output buffer
    ///
    /// This is going to be used for several cases:
    ///
    /// 1. `Flush(0)` before closing the connection
    /// 2. `Flush(0)` to before receiving new request (if needed)
    /// 3. `Flush(N)` to wait when you can continue producing some data, this
    ///    allows TCP pushback. To be able not to put everything in output
    ///    buffer at once. Still probably more efficient than `Flush(0)`
    Flush(usize),
    /// Wait until deadline
    ///
    /// This useful for two cases:
    ///
    /// 1. Just wait before doing anything if required by business logic
    /// 2. Wait until `wakeup` happens or atimeout whatever comes first
    Sleep,
}

pub trait Protocol<C, S: StreamSocket>: Sized {
    type Seed: Any+Sized;
    /// Starting the protocol (e.g. accepted a socket)
    fn create(seed: Self::Seed, sock: &mut S, scope: &mut Scope<C>)
        -> Request<Self>;

    /// The action WaitBytes or WaitDelimiter is complete
    ///
    /// Note you don't have to consume input buffer. The data is in the
    /// transport, but you are free to ignore it. This may be useful for
    /// example to yield `Bytes(4)` to read the header size and then yield
    /// bigger value to read the whole header at once. But be careful, if
    /// you don't consume bytes you will repeatedly receive them again.
    fn bytes_read(self, transport: &mut Transport<S>,
                  end: usize, scope: &mut Scope<C>)
        -> Request<Self>;

    /// The action Flush is complete
    fn bytes_flushed(self, transport: &mut Transport<S>,
                     scope: &mut Scope<C>)
        -> Request<Self>;

    /// Timeout happened, which means either deadline reached in
    /// Bytes, Delimiter, Flush. Or Sleep has passed.
    fn timeout(self, transport: &mut Transport<S>, scope: &mut Scope<C>)
        -> Request<Self>;

    /// Message received (from the main loop)
    fn wakeup(self, transport: &mut Transport<S>, scope: &mut Scope<C>)
        -> Request<Self>;
}
