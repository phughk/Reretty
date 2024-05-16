use crate::handler_two::Handler;
use socket2::{Domain, Protocol, Socket, Type};
use std::any::Any;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::Poll;

struct TcpClientHandler<A>
where
    A: Handler<TYPE_DOWN = Box<[u8]>, TYPE_UP = dyn Any>,
{
    upper: A,
    socket: Socket,
}

pub fn new<A>() -> TcpClientHandler<A> {
    TcpClientHandler {
        upper: A,
        socket: Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)),
    }
}

impl<A> Handler for TcpClientHandler<A> {
    type TYPE_UP = Box<[u8]>;
    type TYPE_DOWN = ();

    fn set_up<A: Handler<TYPE_DOWN = Self::TYPE_UP, TYPE_UP = dyn Any>>(&mut self, next_up: Rc<A>) {
        self.upper = next_up;
    }

    fn set_down<A: Handler<TYPE_UP = Self::TYPE_DOWN, TYPE_DOWN = dyn Any>>(
        &mut self,
        next_down: Rc<A>,
    ) {
        panic!("Trying to set down handler for TcpHandler")
    }

    fn read_poll(&self) -> Result<Poll<Box<[u8]>>, Box<dyn Error>> {
        self.socket.accept()
    }

    fn read_async(&self) -> Pin<Box<dyn Future<Output = Result<Self::TYPE_UP, Box<dyn Error>>>>> {
        todo!()
    }

    fn write(&self, data: Self::TYPE_DOWN) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn write_async(
        &self,
        data: Self::TYPE_DOWN,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>>>> {
        todo!()
    }
}
