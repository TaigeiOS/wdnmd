use std::ffi::CString;
use crate::{print_log, LogLevel};
use std::process::{Command, exit};
use nix::sys::signal::{signal, Signal, SigHandler, sigprocmask, SigmaskHow, SigSet};
use std::os::raw::c_int;
use std::os::unix::process::CommandExt;
use nix::unistd::{fork, ForkResult};
use exec::Error;

extern "C" fn sig_handler(signal: c_int) {
    print_log(LogLevel::Warn, &*("Received signal ".to_owned() + &*signal.to_string()));
}

pub fn put_env() {
    unsafe {
        libc::putenv(CString::new("HOME=/").unwrap().into_raw());
        libc::putenv(CString::new("PATH=/bin:/sbin:/usr/bin:/usr/sbin:/usr/local/bin").unwrap().into_raw());
        libc::putenv(CString::new("SHELL=/bin/sh").unwrap().into_raw());
    }
}

pub fn block_signals() {
    unsafe {
        signal(Signal::SIGUSR1, SigHandler::Handler(sig_handler));
        signal(Signal::SIGUSR2, SigHandler::Handler(sig_handler));
        signal(Signal::SIGTERM, SigHandler::Handler(sig_handler));
        signal(Signal::SIGQUIT, SigHandler::Handler(sig_handler));
        signal(Signal::SIGINT, SigHandler::Handler(sig_handler));
        signal(Signal::SIGHUP, SigHandler::Handler(sig_handler));
        signal(Signal::SIGTSTP, SigHandler::Handler(sig_handler));
        signal(Signal::SIGSTOP, SigHandler::Handler(sig_handler));
        sigprocmask(SigmaskHow::SIG_BLOCK, Some(&SigSet::all()), None);
    }
}

fn unblock_signals() {
    unsafe {
        signal(Signal::SIGUSR1, SigHandler::SigDfl);
        signal(Signal::SIGUSR2, SigHandler::SigDfl);
        signal(Signal::SIGTERM, SigHandler::SigDfl);
        signal(Signal::SIGQUIT, SigHandler::SigDfl);
        signal(Signal::SIGINT, SigHandler::SigDfl);
        signal(Signal::SIGHUP, SigHandler::SigDfl);
        signal(Signal::SIGTSTP, SigHandler::SigDfl);
        signal(Signal::SIGSTOP, SigHandler::SigDfl);
        sigprocmask(SigmaskHow::SIG_UNBLOCK, Some(&SigSet::all()), None);
    }
}

pub fn run_program(bin: &str) {
    print_log(LogLevel::Info, &*("wdnmd: launching program ".to_owned() + bin));
    let mut args = bin.split(' ').map(|a| {a.to_string()});
    if let Some(c) = args.next() {
        let mut command = Command::new(c);
        for arg in args {
            command.arg(arg);
        }
        unsafe {
            match command
                .pre_exec(|| {unblock_signals(); Ok(())})
                .spawn() {
                Ok(mut child) => {
                    match child.wait() {
                        Ok(_) => {}
                        Err(_) => {
                            print_log(LogLevel::Error, "Failed to wait for program")
                        }
                    }
                }
                _ => {
                    print_log(LogLevel::Error, "Cannot launch specific program")
                }
            }
        }
    }
}

pub fn run_containerd() {
    match unsafe{fork()} {
        Ok(ForkResult::Child) => {
            unblock_signals();
            match exec::Command::new("/bin/dockerd").exec() {
                Error::BadArgument(_) => {
                    exit(0);
                }
                Error::Errno(_) => {
                    print_log(LogLevel::Error, "Error during execution docker daemon");
                    exit(0);
                }
            }
        }
        Ok(ForkResult::Parent { child, .. }) => {
            // todo
        }
        Err(_) => {
            print_log(LogLevel::Fatal, "launch containerd failed");
        }
    }
}
