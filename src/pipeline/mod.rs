use crate::error::PipelineError;
use crate::handler::Handler;
use std::any::Any;
use std::future::Future;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::pin::Pin;

/// A pipeline that is associated with either a client (single session) or a server (multiple sessions)
/// IN type is what the inbound messages look like i.e. end result of reading and processing inbound traffic
/// OUT type is what the outbound messages look like i.e. what we are writing to the network
pub struct Pipeline<'a, IN, OUT, READ: Read, WRITE: Write> {
    // These 2 are just to retain type information
    _in: PhantomData<IN>,
    _out: PhantomData<OUT>,
    handlers: Vec<Box<dyn for<'a> Handler<IN = dyn Any, OUT = dyn Any>>>,
    reader: READ,
    writer: WRITE,
}

impl<'a, IN, OUT, READ: Read, WRITE: Write> Pipeline<'a, IN, OUT, READ, WRITE> {
    pub fn new(capacity: usize, reader: READ, writer: WRITE) -> Pipeline<'a, IN, OUT, READ, WRITE> {
        Pipeline {
            _in: Default::default(),
            _out: Default::default(),
            handlers: Vec::with_capacity(capacity),
            reader,
            writer,
        }
    }

    pub fn handler_add_first<'a>(
        &mut self,
        handler: impl Handler<'a, IN = dyn Any, OUT = dyn Any>,
    ) {
        self.handlers.insert(0, handler);
    }

    pub fn handler_add_last<'a>(&mut self, handler: impl Handler<'a, IN = dyn Any, OUT = dyn Any>) {
        self.handlers.push(handler);
    }

    pub fn write(&self, out: OUT) -> Result<(), (OUT, PipelineError)> {
        Ok(())
    }

    pub fn write_async(
        &self,
        out: OUT,
    ) -> Pin<Box<dyn Future<Output = Result<(), (OUT, PipelineError)>>>> {
        Box::pin(async move { Ok(()) })
    }
}
