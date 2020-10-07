
pub(crate) enum State {
    Closed,
    HalfClosed,
    Open,
}

pub(crate) struct Socket {
    pub(crate) state: State,
}

impl Socket {
    pub(crate) fn create() -> [Socket; 4] {
        [
            Socket::default(),
            Socket::default(),
            Socket::default(),
            Socket::default(),
        ]
    }

    pub(crate) fn is_closed(&self) -> bool {
        matches!(&self.state, State::Closed)
    }

    pub(crate) fn is_open(&self) -> bool {
        matches!(&self.state, State::Open)
    }
}

impl Default for Socket {
    fn default() -> Self {
        Self {
            state: State::Closed,
        }
    }

}
