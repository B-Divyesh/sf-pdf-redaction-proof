//! Platform sandbox entered by the PDF worker after it has received its input.
//!
//! The worker is deliberately given bytes over stdin, never a pathname.  The
//! sandbox is installed before those bytes are parsed, so a parser bug cannot
//! be used to open another file or create a network socket.

use std::io;

#[cfg(target_os = "linux")]
mod platform {
    use super::*;

    const LANDLOCK_CREATE_RULESET_VERSION: libc::c_int = 1;
    const LANDLOCK_ACCESS_FS_EXECUTE: u64 = 1 << 0;
    const LANDLOCK_ACCESS_FS_WRITE_FILE: u64 = 1 << 1;
    const LANDLOCK_ACCESS_FS_READ_FILE: u64 = 1 << 2;
    const LANDLOCK_ACCESS_FS_READ_DIR: u64 = 1 << 3;
    const LANDLOCK_ACCESS_FS_REMOVE_DIR: u64 = 1 << 4;
    const LANDLOCK_ACCESS_FS_REMOVE_FILE: u64 = 1 << 5;
    const LANDLOCK_ACCESS_FS_MAKE_CHAR: u64 = 1 << 6;
    const LANDLOCK_ACCESS_FS_MAKE_DIR: u64 = 1 << 7;
    const LANDLOCK_ACCESS_FS_MAKE_REG: u64 = 1 << 8;
    const LANDLOCK_ACCESS_FS_MAKE_SOCK: u64 = 1 << 9;
    const LANDLOCK_ACCESS_FS_MAKE_FIFO: u64 = 1 << 10;
    const LANDLOCK_ACCESS_FS_MAKE_BLOCK: u64 = 1 << 11;
    const LANDLOCK_ACCESS_FS_MAKE_SYM: u64 = 1 << 12;
    const LANDLOCK_ACCESS_FS_REFER: u64 = 1 << 13;
    const LANDLOCK_ACCESS_FS_TRUNCATE: u64 = 1 << 14;
    const LANDLOCK_ACCESS_FS_IOCTL_DEV: u64 = 1 << 15;

    #[repr(C)]
    struct RulesetAttr {
        handled_access_fs: u64,
        handled_access_net: u64,
        scoped: u64,
    }

    fn deny_filesystem() -> io::Result<()> {
        let version = unsafe {
            libc::syscall(
                libc::SYS_landlock_create_ruleset,
                std::ptr::null::<RulesetAttr>(),
                0,
                LANDLOCK_CREATE_RULESET_VERSION,
            )
        };
        if version < 1 {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Landlock is unavailable on this Linux kernel",
            ));
        }
        let supported = match version {
            1 => (1 << 13) - 1,
            2 => (1 << 14) - 1,
            3 | 4 => (1 << 15) - 1,
            _ => (1 << 16) - 1,
        };
        let attr = RulesetAttr {
            handled_access_fs: supported
                & (LANDLOCK_ACCESS_FS_EXECUTE
                    | LANDLOCK_ACCESS_FS_WRITE_FILE
                    | LANDLOCK_ACCESS_FS_READ_FILE
                    | LANDLOCK_ACCESS_FS_READ_DIR
                    | LANDLOCK_ACCESS_FS_REMOVE_DIR
                    | LANDLOCK_ACCESS_FS_REMOVE_FILE
                    | LANDLOCK_ACCESS_FS_MAKE_CHAR
                    | LANDLOCK_ACCESS_FS_MAKE_DIR
                    | LANDLOCK_ACCESS_FS_MAKE_REG
                    | LANDLOCK_ACCESS_FS_MAKE_SOCK
                    | LANDLOCK_ACCESS_FS_MAKE_FIFO
                    | LANDLOCK_ACCESS_FS_MAKE_BLOCK
                    | LANDLOCK_ACCESS_FS_MAKE_SYM
                    | LANDLOCK_ACCESS_FS_REFER
                    | LANDLOCK_ACCESS_FS_TRUNCATE
                    | LANDLOCK_ACCESS_FS_IOCTL_DEV),
            handled_access_net: 0,
            scoped: 0,
        };
        let fd = unsafe {
            libc::syscall(
                libc::SYS_landlock_create_ruleset,
                &attr,
                std::mem::size_of::<RulesetAttr>(),
                0,
            ) as libc::c_int
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let result = unsafe { libc::syscall(libc::SYS_landlock_restrict_self, fd, 0) };
        unsafe { libc::close(fd) };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    fn deny_network() -> io::Result<()> {
        const BPF_LD: u16 = 0x00;
        const BPF_W: u16 = 0x00;
        const BPF_ABS: u16 = 0x20;
        const BPF_JMP: u16 = 0x05;
        const BPF_JEQ: u16 = 0x10;
        const BPF_K: u16 = 0x00;
        const BPF_RET: u16 = 0x06;
        const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;
        const SECCOMP_RET_ERRNO: u32 = 0x0005_0000;
        let mut filters = vec![libc::sock_filter {
            code: BPF_LD | BPF_W | BPF_ABS,
            jt: 0,
            jf: 0,
            k: 0,
        }];
        for syscall in [
            libc::SYS_socket,
            libc::SYS_socketpair,
            libc::SYS_connect,
            libc::SYS_bind,
            libc::SYS_listen,
            libc::SYS_accept,
            libc::SYS_accept4,
            libc::SYS_sendto,
            libc::SYS_recvfrom,
            libc::SYS_sendmsg,
            libc::SYS_recvmsg,
        ] {
            filters.push(libc::sock_filter {
                code: BPF_JMP | BPF_JEQ | BPF_K,
                jt: 0,
                jf: 1,
                k: syscall as u32,
            });
            filters.push(libc::sock_filter {
                code: BPF_RET | BPF_K,
                jt: 0,
                jf: 0,
                k: SECCOMP_RET_ERRNO | libc::EPERM as u32,
            });
        }
        filters.push(libc::sock_filter {
            code: BPF_RET | BPF_K,
            jt: 0,
            jf: 0,
            k: SECCOMP_RET_ALLOW,
        });
        let program = libc::sock_fprog {
            len: filters.len() as u16,
            filter: filters.as_mut_ptr(),
        };
        let result = unsafe {
            libc::prctl(
                libc::PR_SET_SECCOMP,
                libc::SECCOMP_MODE_FILTER,
                &program as *const libc::sock_fprog,
            )
        };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub fn enter() -> io::Result<()> {
        let result = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
        if result != 0 {
            return Err(io::Error::last_os_error());
        }
        deny_filesystem()?;
        deny_network()
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use std::ffi::{CStr, CString};

    #[link(name = "sandbox")]
    extern "C" {
        fn sandbox_init(
            profile: *const libc::c_char,
            flags: u64,
            error: *mut *mut libc::c_char,
        ) -> libc::c_int;
        fn sandbox_free_error(error: *mut libc::c_char);
    }

    pub fn enter() -> io::Result<()> {
        let profile = CString::new(
            "(version 1)(allow default)(deny network*)(deny file-read*)(deny file-write*)",
        )
        .unwrap();
        let mut error = std::ptr::null_mut();
        let result = unsafe { sandbox_init(profile.as_ptr(), 0, &mut error) };
        if result != 0 {
            let message = if error.is_null() {
                "macOS sandbox initialization failed".into()
            } else {
                let value = unsafe { CStr::from_ptr(error) }
                    .to_string_lossy()
                    .into_owned();
                unsafe { sandbox_free_error(error) };
                value
            };
            return Err(io::Error::other(message));
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        Security::{
            AllocateAndInitializeSid, CreateRestrictedToken, FreeSid, DISABLE_MAX_PRIVILEGE,
            SID_AND_ATTRIBUTES, SID_IDENTIFIER_AUTHORITY, TOKEN_DUPLICATE, TOKEN_IMPERSONATE,
            TOKEN_QUERY,
        },
        System::Threading::{GetCurrentProcess, OpenProcessToken, SetThreadToken},
    };

    pub fn enter() -> io::Result<()> {
        unsafe {
            let mut process_token = std::ptr::null_mut();
            if OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_QUERY | TOKEN_DUPLICATE | TOKEN_IMPERSONATE,
                &mut process_token,
            ) == 0
            {
                return Err(io::Error::last_os_error());
            }
            let authority = SID_IDENTIFIER_AUTHORITY {
                Value: [0, 0, 0, 0, 0, 0],
            };
            let mut null_sid = std::ptr::null_mut();
            if AllocateAndInitializeSid(&authority, 1, 0, 0, 0, 0, 0, 0, 0, 0, &mut null_sid) == 0 {
                CloseHandle(process_token);
                return Err(io::Error::last_os_error());
            }
            let mut restricted_sid = SID_AND_ATTRIBUTES {
                Sid: null_sid,
                Attributes: 0,
            };
            let mut restricted_token = std::ptr::null_mut();
            let created = CreateRestrictedToken(
                process_token,
                DISABLE_MAX_PRIVILEGE,
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
                1,
                &mut restricted_sid,
                &mut restricted_token,
            );
            CloseHandle(process_token);
            FreeSid(null_sid);
            if created == 0 {
                return Err(io::Error::last_os_error());
            }
            let installed = SetThreadToken(std::ptr::null(), restricted_token);
            CloseHandle(restricted_token);
            if installed == 0 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod platform {
    use super::*;
    pub fn enter() -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "no PDF sandbox for this operating system",
        ))
    }
}

pub fn enter() -> io::Result<()> {
    platform::enter()
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    #[test]
    fn claim_document_privacy_sandbox_denies_filesystem_and_network_syscalls() {
        let child = unsafe { libc::fork() };
        assert!(child >= 0);
        if child == 0 {
            if super::enter().is_err() {
                unsafe { libc::_exit(10) };
            }
            let path = b"/etc/passwd\0";
            let fd = unsafe { libc::open(path.as_ptr().cast(), libc::O_RDONLY) };
            let socket = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
            let denied = fd == -1
                && socket == -1
                && std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM);
            unsafe { libc::_exit(if denied { 0 } else { 11 }) };
        }
        let mut status = 0;
        assert_eq!(unsafe { libc::waitpid(child, &mut status, 0) }, child);
        assert!(libc::WIFEXITED(status));
        assert_eq!(libc::WEXITSTATUS(status), 0);
    }
}
