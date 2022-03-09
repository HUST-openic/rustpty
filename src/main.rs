use std::rc::Rc;

use rustcmd::mux::Mux;
use rustcmd::pty::PtySize;
fn main() {
    let mux = Rc::new(Mux::new(PtySize::default()).unwrap());
    Mux::set_mux(&mux);

    mux.start().unwrap();
}