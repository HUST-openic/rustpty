use nix::pty::forkpty;
use nix::unistd::ForkResult;
use nix::unistd::read;
use std::os::unix::io::RawFd;
use std::process::Command;
use std::vec;

// Vec<u8>: UTF8 encoded sequences.

// Read from file descriptor.
fn read_from_fd(fd: RawFd) -> Option<Vec<u8>> {
    // Temp buffer with limited size.
    let mut read_buffer = [0; 65536];

    // Read from file descriptor to the buffer.
    let read_result = read(fd, &mut read_buffer);

    // Match Result to Option.
    match read_result {
        // Truncate buffer size. Only return content.
        Ok(bytes_read) => Some(read_buffer[..bytes_read].to_vec()),
        Err(_) => None,
    }
}

// Spawn pty with shell path.
// Returns raw file descriptor for the shell.
unsafe fn spawn_pty_with_shell(default_shell: String) -> RawFd {
    match forkpty(None, None) {
        // Spawn successful.
        Ok(for_pty_res) => {
            let stdout_fd = for_pty_res.master;
            // If the result is a child process, spawn a new shell.
            if let ForkResult::Child = for_pty_res.fork_result {
                Command::new(&default_shell)
                .spawn()
                .expect("failed to spawn");

                // wait for 2s and then exit.
                std::thread::sleep(std::time::Duration::from_millis(2000));
                std::process::exit(0);
            }
            stdout_fd
        },
        Err(e) => {
            panic!("failed to fork {:?}", e);
        }
    }
}

fn main() {
    // Get default shell path.
    let default_shell = std::env::var("SHELL")
        .expect("could not find default shell from &SHELL"); // /bin/bash
    unsafe {
        // Spawn pty with shell path.
        let stdout_fd = spawn_pty_with_shell(default_shell);

        // Buffer.
        let mut read_buffer: Vec<u8> = vec![];
        
        // Constantly loop through contents if read_from_fd returns content.
        loop {
            match read_from_fd(stdout_fd) {
                // Add to the buffer with contents.
                Some(mut read_bytes) => {
                    read_buffer.append(&mut read_bytes);
                },
                None => {
                    println!("{:?}", String::from_utf8(read_buffer).unwrap());
                    std::process::exit(0);
                }
            }
        }
    }

}
