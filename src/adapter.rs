use heapless::{
    String,
    spsc::{Producer, Consumer},
    consts::*,
};
use crate::protocol::{Request, Response, JoinInfo};

pub struct Adapter<'q> {
    producer: Producer<'q,Request, U1>,
    consumer: Consumer<'q, Response, U1>,

}

impl<'q> Adapter<'q> {
    pub fn new(
        producer: Producer<'q, Request, U1>,
        consumer: Consumer<'q, Response, U1>
    ) -> Self {
        Self {
            producer,
            consumer,
        }
    }

    pub fn join(&mut self, ssid: &str, password: &str) -> Result<Response, ()> {
        self.producer.enqueue(
            Request::Join(JoinInfo::Wep {
                ssid: String::from(ssid),
                password: String::from(password),
            })
        );

        loop {
            if let Some(response) = self.consumer.dequeue() {
                log::info!("response {:?}", response);
                return Ok(response);
            }
        }
    }
}