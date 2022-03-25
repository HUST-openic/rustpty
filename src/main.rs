mod raw_guard;


// TODO: Input should not from stdin but from xtermjs (as String).
// TODO: Better to send String to xtermjs directly instead of looping content.
// TODO: To work with tauri events, additional buffers is needed for both input & output.

// #[cfg(feature = "backend-std")]
mod main {
    use std::io::{Read as _, Write as _};
    use std::os::unix::io::AsRawFd as _;

    pub fn run(child: &pty_process::std::Child) {
        let _raw = super::raw_guard::RawGuard::new();
        let mut buf = [0_u8; 4096];
        let pty = child.pty().as_raw_fd();
        let stdin = std::io::stdin().as_raw_fd();

        loop {
            let mut set = nix::sys::select::FdSet::new();
            set.insert(pty);
            set.insert(stdin);
            match nix::sys::select::select(
                None,
                Some(&mut set),
                None,
                None,
                None,
            ) {
                Ok(n) => {
                    if n > 0 {
                        if set.contains(pty) {
                            match child.pty().read(&mut buf) {
                                Ok(bytes) => {
                                    let buf = &buf[..bytes];
                                    let stdout = std::io::stdout();
                                    let mut stdout = stdout.lock();
                                    stdout.write_all(buf).unwrap();
                                    stdout.flush().unwrap();
                                }
                                Err(e) => {
                                    // EIO means that the process closed the other
                                    // end of the pty
                                    if e.raw_os_error() != Some(libc::EIO) {
                                        eprintln!("pty read failed: {:?}", e);
                                    }
                                    break;
                                }
                            };
                        }
                        if set.contains(stdin) {
                            match std::io::stdin().read(&mut buf) {
                                Ok(bytes) => {
                                    let buf = &buf[..bytes];
                                    child.pty().write_all(buf).unwrap();
                                }
                                Err(e) => {
                                    eprintln!("stdin read failed: {:?}", e);
                                    break;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("select failed: {:?}", e);
                    break;
                }
            }
        }
    }

    pub fn run_with_result(child: &pty_process::std::Child) {
        let _raw = super::raw_guard::RawGuard::new();
        let mut buf = [0_u8; 4096];
        let pty = child.pty().as_raw_fd();
        let stdin = std::io::stdin().as_raw_fd();
        let mut result: Vec<u8> = vec![];

        loop {
            let mut set = nix::sys::select::FdSet::new();
            set.insert(pty);
            set.insert(stdin);
            match nix::sys::select::select(
                None,
                Some(&mut set),
                None,
                None,
                None,
            ) {
                Ok(n) => {
                    if n > 0 {
                        if set.contains(pty) {
                            match child.pty().read(&mut buf) {
                                Ok(bytes) => {
                                    let buf = &buf[..bytes];
                                    let stdout = std::io::stdout();
                                    let mut stdout = stdout.lock();
                                    // println!("{}", String::from_utf8(buf.to_vec()).unwrap());
                                    result.extend(buf.to_vec());
                                    stdout.write_all(buf).unwrap();
                                    stdout.flush().unwrap();
                                }
                                Err(e) => {
                                    // EIO means that the process closed the other
                                    // end of the pty
                                    if e.raw_os_error() != Some(libc::EIO) {
                                        eprintln!("pty read failed: {:?}", e);
                                    }
                                    break;
                                }
                            };
                        }
                        if set.contains(stdin) {
                            match std::io::stdin().read(&mut buf) {
                                Ok(bytes) => {
                                    let buf = &buf[..bytes];
                                    child.pty().write_all(buf).unwrap();
                                }
                                Err(e) => {
                                    eprintln!("stdin read failed: {:?}", e);
                                    break;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("select failed: {:?}", e);
                    break;
                }
            }
            println!("{}", String::from_utf8(result.clone()).unwrap());
        }

    }
}

// #[cfg(feature = "backend-std")]
fn main() {
    use pty_process::Command as _;
    use std::os::unix::process::ExitStatusExt as _;

    let mut child = std::process::Command::new("python3")
        // .args(&["500"])
        .spawn_pty(Some(&pty_process::Size::new(24, 80)))
        .unwrap();

    main::run(&child);

    let status = child.wait().unwrap();
    std::process::exit(
        status
            .code()
            .unwrap_or_else(|| status.signal().unwrap_or(0) + 128),
    );
}