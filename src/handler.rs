use std::pin::Pin;
use std::rc::Rc;

/// A trait allowing for communication between 2 handlers
pub trait Handler {
    /// IN type is what inbound messages look like, i.e. what this handlers consumes from the network
    type IN;

    /// OUT type is what outbound messages look like, i.e. what this handlers sends further on in the direction of the network
    type OUT;

    /// Write to the next layer down
    fn write(
        &self,
        outbound: Rc<Self::OUT>,
    ) -> Result<Rc<Self::IN>, (Rc<Self::OUT>, LayerTranslatorError)>;

    /// Write to the next layer down async
    fn write_async(
        &self,
        outbound: &mut Self::OUT,
    ) -> Pin<Box<Result<Rc<Self::IN>, (Rc<Self::OUT>, LayerTranslatorError)>>>;

    /// Read inbound traffic and translate to outbound traffic
    fn read(
        &self,
        inbound: Rc<Self::IN>,
    ) -> Result<Rc<Self::OUT>, (Rc<Self::IN>, LayerTranslatorError)>;

    /// Read inbound traffic and translate to outbound traffic async
    fn read_async(
        &self,
        inbound: Rc<Self::IN>,
    ) -> Pin<Box<Result<Rc<Self::OUT>, (Rc<Self::IN>, LayerTranslatorError)>>>;
}

pub enum LayerTranslatorError {}
