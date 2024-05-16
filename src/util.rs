use std::cmp::min;
use std::io::{Error, ErrorKind, Read, Write};
use std::sync::mpsc::{channel, Receiver, RecvError, Sender};

/// An artificial way of faking a network
///
/// Everything is written from the perspective of the NETWORK, not the pipeline
/// So the inbound channel is what the network will receive
/// The outbound channel is what the network will send
pub struct FakeNetwork<IN: AsRef<[u8]>, OUT: AsRef<[u8]> + From<[u8]>> {
    inbound_send: Sender<IN>,
    inbound_recv: Receiver<IN>,
    outbound_send: Sender<OUT>,
    outbound_recv: Receiver<OUT>,

    out_buf: Vec<u8>,
    in_buf: Vec<u8>,
}

impl<IN, OUT> FakeNetwork<IN, OUT> {
    pub fn new() -> FakeNetwork<IN, OUT> {
        // What the network sends
        let inbound = channel();
        // What the network receives
        let outbound = channel();
        FakeNetwork {
            inbound_send: inbound.0,
            inbound_recv: inbound.1,
            outbound_send: outbound.0,
            outbound_recv: outbound.1,
            out_buf: Vec::new(),
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

impl<IN, OUT> Read for FakeNetwork<IN, OUT> {
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

impl<IN, OUT> Write for FakeNetwork<IN, OUT> {
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

#[cfg(test)]
mod test {
    use crate::util::FakeNetwork;
    use std::io::Read;

    #[test]
    fn network_can_broadcast() {
        let mut network = FakeNetwork::new();
        network.send("The");
        network.send("Sequence");
        network.send("Of");
        network.send("Messages");

        let mut buf = [0u8; 10];
        let copied = network.read(&mut buf).unwrap();
        let mut expected = [0u8; 10];
        expected[0..3].copy_from_slice("The".as_bytes());
        assert_eq!(buf, expected);

        let mut buf = [0u8; 10];
        let copied = network.read(&mut buf).unwrap();
        let mut expected = [0u8; 10];
        expected[0..8].copy_from_slice("Sequence".as_bytes());
        assert_eq!(buf, expected);

        let mut buf = [0u8; 10];
        let copied = network.read(&mut buf).unwrap();
        let mut expected = [0u8; 10];
        expected[0..2].copy_from_slice("Of".as_bytes());
        assert_eq!(buf, expected);

        let mut buf = [0u8; 10];
        let copied = network.read(&mut buf).unwrap();
        let mut expected = [0u8; 10];
        expected[0..8].copy_from_slice("Messages".as_bytes());
        assert_eq!(buf, expected);
    }
}
