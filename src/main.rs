use nix::pty::{openpty};
use nix::unistd::read;
use std::os::unix::io::RawFd;
use std::process::{Command, Child};
use std::vec;
use std::fs::File;
use std::io::{Write, Read};
use std::os::unix::io::FromRawFd;
use std::process::Stdio;

// Vec<u8>: UTF8 encoded sequences.

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
            // Get input from the slave file descriptor.
            .stdin(Stdio::from_raw_fd(slave))
            // .stdout(Stdio::from_raw_fd(slave))
            // .stderr(Stdio::from_raw_fd(slave))
            .spawn()
            .expect("failed to spawn");

            // println!("{:?}", builder.stdin);
            // println!("{:?}", builder.stdout);
            // println!("{:?}", builder.stderr);

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
fn pty_execute(master_file: &mut File, command: &str) {
    // Change the file buffer.
    write!(master_file, "{}", command).unwrap();
    // Write it out.
    master_file.flush().unwrap();
}

fn read_from_master(master_file: &mut File) {
    let mut read_buffer = String::new();
    master_file.read_to_string(&mut read_buffer).unwrap();
    println!("master file descriptor: {}", read_buffer);
}

fn main() {
    let command1 = "touch /home/hzb/Desktop/itworks\n";
    let command2 = "echo hzb\n";

    // Get default shell path.
    let default_shell = std::env::var("SHELL")
        .expect("could not find default shell from &SHELL"); // /bin/bash
    unsafe {
        // Spawn pty with shell path.
        let bidirect_pty = spawn_pty_with_shell(default_shell);

        let mut master_file = File::from_raw_fd(bidirect_pty.mfd);
        // let mut slave_file = File::from_raw_fd(bidirect_pty.sfd);
  
        // read_from_master(&mut master_file.try_clone().unwrap());

        std::thread::sleep(std::time::Duration::from_secs(1));

        pty_execute(&mut master_file, command2);
        
        read_from_master(&mut master_file);

        pty_execute(&mut master_file, command1);


        // // Get the command user input.
        // // For example: only "touch /home/hzb/Desktop/itworks".
        // let mut display = String::new();
        // master_file.read_to_string(&mut display).unwrap();
        // println!("master file descriptor: {}", display);

        std::process::exit(0);


        // Buffer.
        // let mut read_buffer: Vec<u8> = vec![];
        
        // // Constantly loop through contents if read_from_fd returns content.
        // loop {
        //     match read_from_fd(master_pty.mfd) {
        //         // Add to the buffer with contents.
        //         Some(mut read_bytes) => {
        //             read_buffer.append(&mut read_bytes);
        //             println!("Some");
        //         },
        //         None => {
        //             println!("{:?}", String::from_utf8(read_buffer).unwrap());
        //             std::process::exit(0);
        //         }
        //     }
        // }
    }
}
