use std::rc::Rc;

use rustcmd::mux::Mux;
use rustcmd::pty::PtySize;
fn main() {
    // Create.
    let mux = Rc::new(Mux::new(PtySize::default()).unwrap());
    Mux::set_mux(&mux);

    // Start.
    mux.start().unwrap();
}