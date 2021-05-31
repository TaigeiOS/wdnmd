mod unsafe_ops;
mod fs;

use nix::unistd;
use std::process::{Command, exit};
use nix::mount;
use nix::mount::MsFlags;
use nix::sys::stat::{mknod, SFlag, Mode, makedev};
use crate::unsafe_ops::{put_env, block_signals, run_containerd, run_program};
use nix::unistd::Pid;
use crate::fs::mount_from_fstab;

// Consts

enum LogLevel {
    Info,
    Warn,
    Error,
    Fatal
}

fn print_log(level: LogLevel, log: &str) {
    match level {
        LogLevel::Info => {
            print!("[ Info ] ");
        }
        LogLevel::Warn => {
            print!("[ Warn ] ");
        }
        LogLevel::Error => {
            print!("[ Error] ");
        }
        LogLevel::Fatal => {
            print!("[ Fatal] ");
        }
    }
    println!("{}", log);
}

fn fatal(info: &str) {
    print_log(LogLevel::Fatal, info);
    print_log(LogLevel::Fatal, "Trying drop you into rescue shell...");
    let res = Command::new("/bin/sh").spawn();
    match res {
        Ok(mut r) => {
            r.wait();
        }
        Err(_) => {
            print_log(LogLevel::Fatal, "Cannot launch emergency shell, quitting.");
            exit(-1);
        }
    }
}

fn main() {
    let procfs_flags: MsFlags = MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC | MsFlags::MS_RELATIME;
    let devfs_flags: MsFlags = MsFlags::MS_NOSUID | MsFlags::MS_RELATIME;
    let sysfs_flags: MsFlags = MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC | MsFlags::MS_RELATIME;

    let usr_perm_6: Mode = Mode::S_IWUSR | Mode::S_IRUSR;
    let grp_perm_2: Mode = Mode::S_IRGRP;
    let grp_perm_6: Mode = Mode::S_IWGRP | Mode::S_IRGRP;
    let oth_perm_6: Mode = Mode::S_IWOTH | Mode::S_IROTH;
    print_log(LogLevel::Info, "Init process started");
    let result = unistd::setsid();
    let pid;
    match result {
        Ok(r) => {
            pid = r;
            // Warn if it's not pid 1
            if pid != Pid::from_raw(1) {
                print_log(LogLevel::Warn, "Not pid 1.")
            }
        }
        Err(_) => {
            print_log(LogLevel::Warn, "Cannot get pid.");
        }
    }
    // Set environment
    put_env();
    block_signals();
    // Mount special file systems
    mount::mount(Some("proc"), "/proc", Some("proc"), procfs_flags, Some("mode=0555"));
    mount::mount(Some("dev"), "/dev", Some("dev"), devfs_flags, Some("mode=0755"));
    mount::mount(Some("sys"), "/sys", Some("sys"), sysfs_flags, Some("mode=0555"));

    // Make node
    mknod("/dev/tty", SFlag::S_IFCHR, usr_perm_6, makedev(5, 0));
    mknod("/dev/tty1", SFlag::S_IFCHR, usr_perm_6 | grp_perm_2, makedev(4, 1));
    mknod("/dev/tty2", SFlag::S_IFCHR, usr_perm_6 | grp_perm_2, makedev(4, 2));
    mknod("/dev/console", SFlag::S_IFCHR, usr_perm_6 | grp_perm_6 | oth_perm_6, makedev(5, 1));
    mknod("/dev/null", SFlag::S_IFCHR, usr_perm_6 | grp_perm_6 | oth_perm_6, makedev(1, 3));
    mknod("/dev/kmsg", SFlag::S_IFCHR, usr_perm_6 | grp_perm_6, makedev(1, 11));

    // Mount other fs
    mount_from_fstab();
    // Run container
    run_containerd();
    // Run tty
    run_program("/bin/getty");
}
