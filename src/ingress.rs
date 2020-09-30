use crate::buffer::Buffer;

pub struct Ingress {
    buffer: Buffer,
}

impl Ingress {
    pub(crate) fn new() -> Self {
        Ingress {
            buffer: Buffer::new()
        }
    }
}
