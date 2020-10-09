use drogue_network::tcp::Mode;

pub(crate) enum State {
    Closed,
    HalfClosed,
    Open,
}

pub(crate) struct Socket {
    pub(crate) state: State,
    pub(crate) mode: Mode,
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

    pub(crate) fn is_blocking(&self) -> bool {
        matches!(&self.mode, Mode::Blocking)
    }
}

impl Default for Socket {
    fn default() -> Self {
        Self {
            state: State::Closed,
            mode: Mode::Blocking,
        }
    }

}
