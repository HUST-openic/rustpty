use nix::pty::forkpty;
use nix::unistd::ForkResult;
use std::os::unix::io::RawFd;
use std::process::Command;

fn main() {
    println!("Hello World");
}
