use crate::pty::{Child, MasterPty};
use std::cell::{RefCell, RefMut};

pub struct Tab {
    process: RefCell<Box<dyn Child>>,
    pty: RefCell<Box<dyn MasterPty>>,
    can_close: bool,
}

impl Tab {
    //TODO: use this.
    pub fn writer(&self) -> RefMut<dyn std::io::Write> {
        self.pty.borrow_mut()
    }

    pub fn reader(&self) -> anyhow::Result<Box<dyn std::io::Read + Send>> {
        self.pty.borrow_mut().try_clone_reader()
    }

    pub fn close(&mut self) {
        self.can_close = true;
    }

    pub fn can_close(&self) -> bool {
        self.can_close || self.is_dead()
    }

    pub fn is_dead(&self) -> bool {
        if let Ok(None) = self.process.borrow_mut().try_wait() {
            false
        } else {
            true
        }
    }

    pub fn new(process: Box<dyn Child>, pty: Box<dyn MasterPty>) -> Self {
        Self {
            process: RefCell::new(process),
            pty: RefCell::new(pty),
            can_close: false,
        }
    }
}

impl Drop for Tab {
    fn drop(&mut self) {
        self.process.borrow_mut().kill().ok();
        self.process.borrow_mut().wait().ok();
    }
}
