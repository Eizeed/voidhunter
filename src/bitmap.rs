#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BitmapU16 {
    inner: u16,
}

impl From<u16> for BitmapU16 {
    fn from(value: u16) -> Self {
        BitmapU16 { inner: value }
    }
}

#[allow(dead_code)]
impl BitmapU16 {
    const FRONTIER: u16 = 0b0001;
    const AGENTS: u16 = 0b0010;
    const HP: u16 = 0b0100;
    const INGAME_TIMER: u16 = 0b1000;

    const RES_TIMER: u16 = 0b0001_0000;
    const PAUSE: u16 = 0b0010_0000;
    const CONFIRM_DIALOG: u16 = 0b0100_0000;
    const CHALLENGES: u16 = 0b1000_0000;

    const LOADING: u16 = 0b0001_0000_0000;

    pub fn frontier(&self) -> bool {
        (self.inner & Self::FRONTIER) != 0
    }

    pub fn agents(&self) -> bool {
        (self.inner & Self::AGENTS) != 0
    }

    pub fn hp(&self) -> bool {
        (self.inner & Self::HP) != 0
    }

    pub fn ingame_timer(&self) -> bool {
        (self.inner & Self::INGAME_TIMER) != 0
    }

    pub fn res_timer(&self) -> bool {
        (self.inner & Self::RES_TIMER) != 0
    }

    pub fn pause(&self) -> bool {
        (self.inner & Self::PAUSE) != 0
    }

    pub fn confirm_dialog(&self) -> bool {
        (self.inner & Self::CONFIRM_DIALOG) != 0
    }

    pub fn challenges(&self) -> bool {
        (self.inner & Self::CHALLENGES) != 0
    }

    pub fn loading(&self) -> bool {
        (self.inner & Self::LOADING) != 0
    }

    pub fn set_frontier(&mut self, val: bool) {
        if val {
            self.inner |= Self::FRONTIER;
        } else {
            self.inner &= !Self::FRONTIER;
        }
    }
    pub fn set_agents(&mut self, val: bool) {
        if val {
            self.inner |= Self::AGENTS;
        } else {
            self.inner &= !Self::AGENTS;
        }
    }
    pub fn set_hp(&mut self, val: bool) {
        if val {
            self.inner |= Self::HP;
        } else {
            self.inner &= !Self::HP;
        }
    }
    pub fn set_ingame_timer(&mut self, val: bool) {
        if val {
            self.inner |= Self::INGAME_TIMER;
        } else {
            self.inner &= !Self::INGAME_TIMER;
        }
    }
    pub fn set_res_timer(&mut self, val: bool) {
        if val {
            self.inner |= Self::RES_TIMER;
        } else {
            self.inner &= !Self::RES_TIMER;
        }
    }
    pub fn set_pause(&mut self, val: bool) {
        if val {
            self.inner |= Self::PAUSE;
        } else {
            self.inner &= !Self::PAUSE;
        }
    }
    pub fn set_confirm_dialog(&mut self, val: bool) {
        if val {
            self.inner |= Self::CONFIRM_DIALOG;
        } else {
            self.inner &= !Self::CONFIRM_DIALOG;
        }
    }
    pub fn set_challenges(&mut self, val: bool) {
        if val {
            self.inner |= Self::CHALLENGES;
        } else {
            self.inner &= !Self::CHALLENGES;
        }
    }

    pub fn set_loading(&mut self, val: bool) {
        if val {
            self.inner |= Self::LOADING;
        } else {
            self.inner &= !Self::LOADING;
        }
    }
}
