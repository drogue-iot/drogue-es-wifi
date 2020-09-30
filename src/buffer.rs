

pub(crate) struct Buffer {
    pos: usize,
    buffer: [u8; 4096],
}

impl Buffer {

    pub(crate) fn new() -> Self {
        Buffer {
            pos: 0,
            buffer: [0; 4096],
        }
    }

    pub(crate) fn write(&mut self, data: &[u8]) -> Result<&[u8],()> {
        Ok(&[])
    }

}