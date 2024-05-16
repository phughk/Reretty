mod tcp_handler;

use std::any::Any;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Poll;

pub trait Handler {
    type TYPE_UP;
    type TYPE_DOWN;

    /// Set the next handler up, which consumes from this handler on inbound and
    /// produces to this handler on outbound
    fn set_up<A: Handler<TYPE_DOWN = Self::TYPE_UP, TYPE_UP = dyn Any>>(&mut self, next_up: Rc<A>);

    fn set_down<A: Handler<TYPE_UP = Self::TYPE_DOWN, TYPE_DOWN = dyn Any>>(
        &mut self,
        next_down: Rc<A>,
    );

    /// Read from the handler, which will invoke from lower layers
    /// This is synchronous, but should be low-blocking
    /// low-blocking because near the OS layer it may be blocking
    fn read_poll(&self) -> Result<Poll<Self::TYPE_UP>, Box<dyn Error>>;

    fn read_async(&self) -> Pin<Box<dyn Future<Output = Result<Self::TYPE_UP, Box<dyn Error>>>>>;

    /// Write
    fn write(&self, data: Self::TYPE_DOWN) -> Result<(), Box<dyn Error>>;

    /// Write async
    fn write_async(
        &self,
        data: Self::TYPE_DOWN,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>>>>;
}
