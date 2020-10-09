use drogue_network::tcp::Mode;

pub(crate) enum State {
    Closed,
    Open,
    Connected,
    HalfClosed,
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

    pub(crate) fn is_connected(&self) -> bool {
        matches!(&self.state, State::Connected)
    }

    pub(crate) fn is_closed(&self) -> bool {
        matches!(&self.state, State::Closed) || matches!(&self.state, State::HalfClosed)
    }

    pub(crate) fn is_open(&self) -> bool {
        matches!(&self.state, State::Open) || self.is_connected()
    }

    pub(crate) fn is_blocking(&self) -> bool {
        matches!(&self.mode, Mode::Blocking)
    }

    pub(crate) fn is_non_blocking(&self) -> bool {
        matches!(&self.mode, Mode::NonBlocking)
    }

    pub(crate) fn is_timeout(&self) -> bool {
        matches!(&self.mode, Mode::Timeout(_))
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
