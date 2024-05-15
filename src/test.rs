use crate::handler::{Handler, LayerTranslatorError};
use crate::pipeline::Pipeline;
use std::cmp::{max, min};
use std::io::{Error, ErrorKind, Read, Write};
use std::marker::PhantomData;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::mpsc::{channel, Receiver, RecvError, Sender};

#[test]
fn can_send() {
    let fake_network: FakeNetwork<Box<[u8]>, Box<[u8]>> = FakeNetwork::new();
    // We RC the channel because we need mutable borrows to read and write
    let fake_network = Rc::new(fake_network);

    let mut pipeline = Pipeline::new(1, fake_network.clone(), fake_network.clone());
    pipeline.handler_add_first(Doubler::new());
    pipeline.write("one").unwrap();

    assert_eq!(fake_network.recv().unwrap(), "oonnee".as_bytes())
}

struct Doubler<'a> {
    _self_lifetime: PhantomData<&'a ()>,
}

impl<'a> Doubler<'a> {
    pub fn new() -> Doubler<'a> {
        Doubler {
            _self_lifetime: Default::default(),
        }
    }
}

impl<'a> Handler for Doubler<'a> {
    type IN = &'a [u8];
    type OUT = &'a [u8];

    fn write(&self, outbound: Self::OUT) -> Result<Self::IN, (Self::OUT, LayerTranslatorError)> {
        let mut data: Vec<u8> = Vec::with_capacity(outbound.len() * 2);
        for byte in outbound {
            data.push(*byte);
        }
        Ok(data.as_slice())
    }

    fn write_async(
        &self,
        outbound: Self::OUT,
    ) -> Pin<Box<Result<Self::IN, (Self::OUT, LayerTranslatorError)>>> {
        todo!()
    }

    fn read(&self, inbound: Self::IN) -> Result<Self::OUT, (Self::IN, LayerTranslatorError)> {
        let mut data = Vec::with_capacity(inbound.len() / 2);
        for i in 0..inbound.len() / 2 {
            data.push(inbound[i * 2]);
        }
        Ok(data.as_slice())
    }

    fn read_async(
        &self,
        inbound: Self::IN,
    ) -> Pin<Box<Result<Self::OUT, (Self::IN, LayerTranslatorError)>>> {
        todo!()
    }
}

/// An artificial way of faking a network
///
/// Everything is written from the perspective of the NETWORK, not the pipeline
/// So the inbound channel is what the network will receive
/// The outbound channel is what the network will send
pub struct FakeNetwork<'a, IN: AsRef<[u8]>, OUT: AsRef<[u8]> + From<[u8]>> {
    inbound_send: Sender<IN>,
    inbound_recv: Receiver<IN>,
    outbound_send: Sender<OUT>,
    outbound_recv: Receiver<OUT>,

    out_buf: Option<&'a [u8]>,
    in_buf: Vec<u8>,
}

impl<'a, IN, OUT> FakeNetwork<'a, IN, OUT> {
    pub fn new() -> FakeNetwork<'a, IN, OUT> {
        // What the network sends
        let inbound = channel();
        // What the network receives
        let outbound = channel();
        FakeNetwork {
            inbound_send: inbound.0,
            inbound_recv: inbound.1,
            outbound_send: outbound.0,
            outbound_recv: outbound.1,
            out_buf: None,
            in_buf: Vec::with_capacity(1024),
        }
    }

    pub fn send(&self, msg_in: IN) {
        self.inbound_send.send(msg_in).unwrap()
    }

    pub fn recv(&self) -> Result<OUT, RecvError> {
        self.outbound_recv.recv()
    }
}

impl<'a, IN, OUT> Read for FakeNetwork<'a, IN, OUT> {
    /// Read from the fake network
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.out_buf.is_none() {
            self.out_buf = Some(self.inbound_recv.recv().unwrap())
        }
        let res = match self.out_buf {
            None => Err(Error::new(
                ErrorKind::UnexpectedEof,
                "No data in inbound buffer",
            )),
            Some(inner_buf) => {
                let max_index = min(buf.len(), inner_buf.len());
                buf[0..max_index].copy_from_slice(&inner_buf[0..max_index]);
                Ok(max_index)
            }
        };
        match &res {
            Ok(sz) => {
                self.out_buf = self.out_buf.map(|buf| *buf[0..*sz]);
            }
            Err(e) => {}
        }
        res
    }
}

impl<'a, IN, OUT> Write for FakeNetwork<'a, IN, OUT> {
    /// Write to the fake network
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let size_to_copy = self.in_buf.capacity() - self.in_buf.len();
        for byte in buf[0..size_to_copy] {
            self.in_buf.push(byte);
        }
        // TODO this is wrong, what is the supposed size? What about remainder? At capacity?
        let deserialised = IN::<&[u8]>::from(self.in_buf.as_slice());
        self.inbound_send.send(deserialised).unwrap();
        self.in_buf.clear();
        Ok(size_to_copy)
    }

    /// Flush to the fake network
    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}
