use std::io::Write;
use std::marker::PhantomData;
use std::pin::Pin;
use std::rc::Rc;

use crate::handler::{Handler, LayerTranslatorError};
use crate::pipeline::Pipeline;
use crate::util::FakeNetwork;

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
