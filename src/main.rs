use nix::pty::{openpty};
use nix::unistd::read;
use std::fmt::format;
use std::os::unix::io::RawFd;
use std::process::{Command, Child};
use std::vec;
use std::fs::File;
use std::io::{Write, Read};
use std::os::unix::io::FromRawFd;
use std::process::Stdio;

// Vec<u8>: UTF8 encoded sequences.

// The Process for pty:
// Set process stdin & stdout & stderr as the slave (shell).
// The terminal emulator have the master file.
// We write content to the master file. The master file will send it to the slave file as input for the process.
// The shell process will execute the command and return the output to slave file.
// The slave file will then have it's content pushed to the master file. We read from it for display.s

struct bipty {
    process: Child,
    mfd: RawFd,
    sfd: RawFd,
}

// Read from file descriptor.
fn read_from_fd(fd: RawFd) -> Option<Vec<u8>> {
    // Temp buffer with limited size.
    let mut read_buffer = [0; 65536];

    // Read from file descriptor to the buffer.
    let read_result = read(fd, &mut read_buffer);
    println!("read");
    // Match Result to Option.
    match read_result {
        // Truncate buffer size. Only return content.
        Ok(bytes_read) => Some(read_buffer[..bytes_read].to_vec()),
        Err(_) => None,
    }
}

// Spawn pty with shell path.
// Returns raw file descriptor for the shell.
unsafe fn spawn_pty_with_shell(default_shell: String) -> bipty {
    match openpty(None, None) {
        // Spawn successful.
        Ok(pty_res) => {
            let master = pty_res.master;
            let slave = pty_res.slave;
            println!("master fd: {}, slave fd: {}", &master, &slave);
            // If the result is a child process, spawn a new shell.
            let builder = Command::new(&default_shell)
            .stdin(Stdio::from_raw_fd(slave))
            .stdout(Stdio::from_raw_fd(slave))
            .stderr(Stdio::from_raw_fd(slave))
            .spawn()
            .expect("failed to spawn");

            // wait for 2s and then exit.
            std::thread::sleep(std::time::Duration::from_millis(2000));

            bipty {
                process: builder,
                mfd: master,
                sfd: slave,
            }
        },
        Err(e) => {
            panic!("failed to fork {:?}", e);
        }
    }
}

// Execute a command with user input, by flushing master file descripter.
fn pty_execute(mfd: RawFd, command: &str) {
    let mut master_file = unsafe { File::from_raw_fd(mfd) };
    // Write command to the file descriptor.
    write!(master_file, "{}", command).unwrap();
    master_file.flush().unwrap();
}

// Error: Bad file descriptor code 9.
// fn read_from_master(mfd: RawFd) -> String {
//     let mut master_file = unsafe { File::from_raw_fd(mfd) };

//     let mut read_buffer = String::new();
//     master_file.read_to_string(&mut read_buffer).unwrap(); // Execution stops here.
//     format!("Master file content: {}", read_buffer)
// }

fn read_from_master_fd(mfd: RawFd) -> String {
    let mut read_buffer: Vec<u8> = vec![];

    loop {
        match read_from_fd(mfd) {
            Some(mut read_bytes) => {
                read_buffer.append(&mut read_bytes);
            }
            None => {
                break;
            }
        }
    }

    format!("{:?}", String::from_utf8(read_buffer).unwrap())
}

//TODO: Read from the master file without pending the terminal or panic with Bad file desciptor code 9.
fn main() {
    let command1 = "touch /home/hzb/Desktop/itworks\n";
    let command2 = "echo hzb\n";

    // Get default shell path.
    let default_shell = std::env::var("SHELL")
        .expect("could not find default shell from &SHELL"); // /bin/bash
    unsafe {
        // Spawn pty with shell path.
        let bidirect_pty = spawn_pty_with_shell(default_shell);

        // let mut slave_file = File::from_raw_fd(bidirect_pty.sfd);
  
        // read_from_master(&mut master_file.try_clone().unwrap());

        std::thread::sleep(std::time::Duration::from_secs(1));

        let content: String = read_from_master_fd(bidirect_pty.mfd);

        pty_execute(bidirect_pty.mfd, command2);
        
        println!("{}", content);

        // pty_execute(bidirect_pty.mfd, command1);

        std::process::exit(0);
    }
}
