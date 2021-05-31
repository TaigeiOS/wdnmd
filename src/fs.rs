use crate::print_log;
use crate::LogLevel;
use nix::mount::mount;
use nix::mount::MsFlags;
use nix::sys::stat;

pub fn mount_from_fstab() {
    let fs = fstab::FsTab::new("/etc/fstab".as_ref());
    match fs.get_entries() {
        Ok(l) => {
            for i in l {
                let src = i.fs_spec;
                let dst = i.mountpoint;
                let fs_type = i.vfs_type;
                let opts = i.mount_options;
                let mut mount_flags = MsFlags::empty();
                for i in opts {
                    match i.as_str() {
                        "ro" => {
                            mount_flags |= MsFlags::MS_RDONLY
                        }
                        "sync" => {
                            mount_flags |= MsFlags::MS_SYNCHRONOUS
                        }
                        "noexec" => {
                            mount_flags |= MsFlags::MS_NOEXEC
                        }
                        "nodev" => {
                            mount_flags |= MsFlags::MS_NODEV
                        }
                        "nosuid" => {
                            mount_flags |= MsFlags::MS_NOSUID
                        }
                        "noatime" => {
                            mount_flags |= MsFlags::MS_NOATIME
                        }
                        "nodiratime" => {
                            mount_flags |= MsFlags::MS_NODIRATIME
                        }
                        "relatime" => {
                            mount_flags |= MsFlags::MS_RELATIME
                        }
                        _ => {}
                    }
                }
                nix::unistd::mkdir(src.as_str(), stat::Mode::S_IRWXU);
                mount(Some(src.as_str()), dst.to_str().unwrap(), Some(fs_type.as_str()), mount_flags, Some("mode=620"));
            }
        }
        Err(_) => {
            print_log(LogLevel::Error, "Failed to read fstab")
        }
    }
}