//! Bivdi sandbox — seccomp-BPF and Landlock hardening for the Runtime.
//!
//! The Runtime is the product (`D-003`) and runs on Linux. The fourth clause of
//! Milestone A's exit gate is *"with no code running outside a sandbox."* This
//! crate is the sandbox: a **seccomp** syscall filter plus a **Landlock**
//! filesystem policy, applied to the process before untrusted (agent/WASI)
//! code runs.
//!
//! # Best-effort, honest about limits
//!
//! - **Seccomp** is applied in "filter mode": an allowlist of syscalls. A
//!   denied syscall kills the process rather than returning an error, which is
//!   the correct fail-closed behaviour for untrusted code.
//! - **Landlock** is applied as a read-only filesystem policy: the process may
//!   read and execute, but may not write, create, delete, truncate, or refer
//!   across the filesystem. This is the closest Linux equivalent to the decided
//!   "no ambient write authority" invariant on the Runtime track.
//! - Both mechanisms are **best-effort**: if the host kernel is too old, or the
//!   container runtime forbids the syscalls (e.g. a seccomp profile that returns
//!   `EPERM`), the sandbox reports *why* it could not engage rather than
//!   silently claiming to be hardened. The caller decides whether that is
//!   acceptable. This is the RFC 0003 principle — never overclaim a guarantee
//!   the platform does not enforce — applied to the Runtime's own claims.
//!
//! # Provisional
//!
//! This is a userspace Linux hardening layer, *not* the kernel-enforced
//! capability boundary of the parked Core track (`P-001` deferred). Unforgeable
//! capabilities are simulated with opaque ids; seccomp/Landlock bound *process*
//! reachability, not capability derivation.

use std::ffi::CString;
use std::io;

/// The result of attempting to engage the sandbox. It is fully hardened only
/// when *both* mechanisms engaged; otherwise it names exactly what failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxReport {
    pub seccomp: Mechanism,
    pub landlock: Mechanism,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mechanism {
    /// The mechanism is engaged and enforcing.
    Engaged,
    /// The mechanism could not be engaged; the reason is the contained string.
    Unavailable(String),
}

impl SandboxReport {
    /// `true` if every mechanism is engaged (fully hardened).
    pub fn is_hardened(&self) -> bool {
        self.seccomp == Mechanism::Engaged && self.landlock == Mechanism::Engaged
    }
}

/// Landlock ABI version, queried at runtime so we degrade gracefully on older
/// kernels instead of failing the whole sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LandlockAbi {
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
}

/// Landlock access-right and rule constants, mirroring the kernel's stable,
/// ABI-locked `linux/landlock.h` values. These are fixed by the Landlock ABI
/// and never change across kernel versions.
mod lnd {
    // Access rights (ABI V1..V5).
    pub const EXECUTE: u64 = 1 << 0;
    pub const WRITE_FILE: u64 = 1 << 1;
    pub const READ_FILE: u64 = 1 << 2;
    pub const READ_DIR: u64 = 1 << 3;
    pub const REMOVE_DIR: u64 = 1 << 4;
    pub const REMOVE_FILE: u64 = 1 << 5;
    pub const MAKE_CHAR: u64 = 1 << 6;
    pub const MAKE_DIR: u64 = 1 << 7;
    pub const MAKE_REG: u64 = 1 << 8;
    pub const MAKE_SOCK: u64 = 1 << 9;
    pub const MAKE_FIFO: u64 = 1 << 10;
    pub const MAKE_BLOCK: u64 = 1 << 11;
    pub const MAKE_SYM: u64 = 1 << 12;
    pub const REFER: u64 = 1 << 13;
    pub const TRUNCATE: u64 = 1 << 14;

    // Rule type (ABI V1).
    pub const RULE_PATH_BENEATH: u32 = 1;

    /// The rights an ABI version handles. `REFER` (V2) and `TRUNCATE` (V3) are
    /// not handled by earlier ABIs and must be omitted or the kernel rejects
    /// the ruleset.
    pub fn handled(abi: super::LandlockAbi) -> u64 {
        use super::LandlockAbi;
        let mut h = EXECUTE
            | WRITE_FILE
            | READ_FILE
            | READ_DIR
            | REMOVE_DIR
            | REMOVE_FILE
            | MAKE_CHAR
            | MAKE_DIR
            | MAKE_REG
            | MAKE_SOCK
            | MAKE_FIFO
            | MAKE_BLOCK
            | MAKE_SYM;
        if matches!(
            abi,
            LandlockAbi::V2 | LandlockAbi::V3 | LandlockAbi::V4 | LandlockAbi::V5 | LandlockAbi::V6
        ) {
            h |= REFER;
        }
        if matches!(
            abi,
            LandlockAbi::V3 | LandlockAbi::V4 | LandlockAbi::V5 | LandlockAbi::V6
        ) {
            h |= TRUNCATE;
        }
        h
    }
}

impl LandlockAbi {
    /// Query the kernel's Landlock ABI version. Per the Landlock API, the ABI
    /// is discovered by calling
    /// `landlock_create_ruleset(NULL, 0, LANDLOCK_CREATE_RULESET_VERSION)`,
    /// which returns the highest supported ABI number (1..=5) — not by trying
    /// each version in turn.
    pub fn query() -> io::Result<LandlockAbi> {
        // LANDLOCK_CREATE_RULESET_VERSION = 1 (the only valid flag for ABI
        // discovery; it is the ABI version of the *call*, and the return value
        // is the ABI of the kernel).
        const LANDLOCK_CREATE_RULESET_VERSION: u32 = 1;

        let rc = unsafe {
            libc::syscall(
                libc::SYS_landlock_create_ruleset,
                std::ptr::null::<libc::c_void>(),
                0usize,
                LANDLOCK_CREATE_RULESET_VERSION,
            )
        };
        if rc < 0 {
            return Err(io::Error::last_os_error());
        }
        // `rc` is a valid ruleset fd (the kernel created a throwaway ruleset to
        // report the ABI) *and* encodes the ABI in its lowest bits per the API,
        // but the canonical interpretation is the return value itself is the ABI
        // when no attr is given. Close the throwaway fd first.
        // `rc` is the ABI version (1..=6), NOT a file descriptor: per the
        // Landlock API, a `landlock_create_ruleset` call with a NULL attr and
        // the VERSION flag returns the kernel ABI directly and must not be
        // closed.
        match rc {
            6 => Ok(LandlockAbi::V6),
            5 => Ok(LandlockAbi::V5),
            4 => Ok(LandlockAbi::V4),
            3 => Ok(LandlockAbi::V3),
            2 => Ok(LandlockAbi::V2),
            1 => Ok(LandlockAbi::V1),
            _ => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "unknown Landlock ABI version",
            )),
        }
    }
}

/// Apply the sandbox. Best-effort: returns a [`SandboxReport`] naming exactly
/// which mechanism engaged and which did not. Each mechanism is a one-way door
/// and can only be engaged once per process; a second call reports the same
/// state.
///
/// # Safety
///
/// Seccomp and Landlock are irreversible for the lifetime of the process. Call
/// this only when the process is ready to drop privileges before running
/// untrusted code.
pub fn engage() -> SandboxReport {
    // Order matters: Landlock must be applied *before* seccomp, because the
    // seccomp allowlist deliberately omits the landlock syscalls (untrusted
    // code must not be able to relax or add Landlock rules).
    let landlock = apply_landlock();
    let seccomp = apply_seccomp();
    SandboxReport { seccomp, landlock }
}

/// Apply a read-only Landlock filesystem policy: the whole filesystem becomes
/// read-only (read + execute allowed; write/create/delete/truncate/refer denied).
fn apply_landlock() -> Mechanism {
    let abi = match LandlockAbi::query() {
        Ok(a) => a,
        Err(e) => return Mechanism::Unavailable(e.to_string()),
    };

    let handled = lnd::handled(abi);

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct RulesetAttr {
        handled_access_fs: u64,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct PathBeneathAttr {
        allowed_access: u64,
        parent_fd: libc::c_int,
    }

    let attr = RulesetAttr {
        handled_access_fs: handled,
    };
    let fd = unsafe {
        libc::syscall(
            libc::SYS_landlock_create_ruleset,
            &attr as *const RulesetAttr as *const libc::c_void,
            std::mem::size_of::<RulesetAttr>(),
            0u32,
        )
    };
    if fd < 0 {
        return Mechanism::Unavailable(io::Error::last_os_error().to_string());
    }

    // Grant read + execute beneath "/" only. Every handled right not granted
    // (i.e. the read-only-denied set) is forbidden.
    //
    // A Landlock path-beneath rule needs a real directory file descriptor, not
    // `AT_FDCWD` (which is only valid for `*at` syscalls).
    let root = CString::new("/").expect("static path");
    let root_fd = unsafe { libc::open(root.as_ptr(), libc::O_PATH | libc::O_CLOEXEC) };
    if root_fd < 0 {
        let e = io::Error::last_os_error();
        unsafe { libc::close(fd as libc::c_int) };
        return Mechanism::Unavailable(e.to_string());
    }

    let allowed = lnd::READ_FILE | lnd::READ_DIR | lnd::EXECUTE;
    let rule = PathBeneathAttr {
        allowed_access: allowed & handled,
        parent_fd: root_fd,
    };
    let rc = unsafe {
        libc::syscall(
            libc::SYS_landlock_add_rule,
            fd,
            lnd::RULE_PATH_BENEATH as libc::c_ulong,
            &rule as *const PathBeneathAttr as *const libc::c_void,
            0u32,
        )
    };
    unsafe { libc::close(root_fd) };
    if rc < 0 {
        let e = io::Error::last_os_error();
        unsafe { libc::close(fd as libc::c_int) };
        return Mechanism::Unavailable(e.to_string());
    }

    let rc = unsafe { libc::syscall(libc::SYS_landlock_restrict_self, fd, 0u32) };
    unsafe { libc::close(fd as libc::c_int) };
    if rc < 0 {
        return Mechanism::Unavailable(io::Error::last_os_error().to_string());
    }

    Mechanism::Engaged
}

/// Apply a seccomp-BPF filter allowing a small, conservative syscall set and
/// failing closed (kill) on anything else.
fn apply_seccomp() -> Mechanism {
    // A conservative allowlist sufficient for the demo CLI and the WASI host:
    // memory, basic I/O, time, thread, and exit. It must include the syscalls
    // Rust's standard library uses even for `println!` (statx, fcntl, clone3,
    // etc.), otherwise the process dies on its own I/O after the filter lands.
    let allowed: &[libc::c_long] = &[
        libc::SYS_read,
        libc::SYS_write,
        libc::SYS_openat,
        libc::SYS_close,
        libc::SYS_fstat,
        libc::SYS_statx,
        libc::SYS_lseek,
        libc::SYS_mmap,
        libc::SYS_mprotect,
        libc::SYS_munmap,
        libc::SYS_brk,
        libc::SYS_rt_sigaction,
        libc::SYS_rt_sigprocmask,
        libc::SYS_sigaltstack,
        libc::SYS_ioctl,
        libc::SYS_fcntl,
        libc::SYS_dup,
        libc::SYS_dup2,
        libc::SYS_dup3,
        libc::SYS_pread64,
        libc::SYS_readv,
        libc::SYS_writev,
        libc::SYS_access,
        libc::SYS_futex,
        libc::SYS_madvise,
        libc::SYS_membarrier,
        libc::SYS_clock_gettime,
        libc::SYS_clock_nanosleep,
        libc::SYS_nanosleep,
        libc::SYS_getrandom,
        libc::SYS_exit,
        libc::SYS_exit_group,
        libc::SYS_rt_sigreturn,
        libc::SYS_getpid,
        libc::SYS_gettid,
        libc::SYS_getcwd,
        libc::SYS_newfstatat,
        libc::SYS_readlinkat,
        libc::SYS_getdents64,
        libc::SYS_prlimit64,
        libc::SYS_set_tid_address,
        libc::SYS_rseq,
        libc::SYS_sched_yield,
        libc::SYS_poll,
        libc::SYS_ppoll,
        libc::SYS_pselect6,
        libc::SYS_epoll_create1,
        libc::SYS_epoll_ctl,
        libc::SYS_epoll_pwait,
        libc::SYS_clone,
        libc::SYS_clone3,
        libc::SYS_execve,
        libc::SYS_wait4,
        libc::SYS_pipe,
        libc::SYS_pipe2,
        libc::SYS_socket,
        libc::SYS_connect,
        libc::SYS_bind,
        libc::SYS_listen,
        libc::SYS_accept,
        libc::SYS_sendto,
        libc::SYS_recvfrom,
        libc::SYS_setsockopt,
        libc::SYS_getsockname,
        libc::SYS_getpeername,
        libc::SYS_shutdown,
        libc::SYS_sched_getaffinity,
        libc::SYS_munlockall,
        libc::SYS_mlock,
        libc::SYS_munlock,
    ];

    // seccomp must be applied only after no-new-privs is set, otherwise a
    // privilege-raising execve could escape the filter.
    let rc = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    if rc < 0 {
        return Mechanism::Unavailable(io::Error::last_os_error().to_string());
    }

    // Classic-BPF program:
    //   LD W ABS 0          ; load syscall nr (offset 0 of seccomp_data)
    //   JEQ allowed[0] -> ALLOW
    //   JEQ allowed[1] -> ALLOW
    //   ...
    //   RET SECCOMP_RET_KILL_PROCESS   (default deny)
    // ALLOW:
    //   RET SECCOMP_RET_ALLOW
    const BPF_LD: u16 = 0x00;
    const BPF_W: u16 = 0x00;
    const BPF_ABS: u16 = 0x20;
    const BPF_JMP: u16 = 0x05;
    const BPF_JEQ: u16 = 0x10;
    const BPF_K: u16 = 0x00;
    const BPF_RET: u16 = 0x06;
    const SECCOMP_RET_KILL_PROCESS: u32 = 0x8000_0000;
    const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;

    let mut insns: Vec<libc::sock_filter> = Vec::with_capacity(allowed.len() + 3);
    // 0: load syscall number.
    insns.push(libc::sock_filter {
        code: BPF_LD | BPF_W | BPF_ABS,
        jt: 0,
        jf: 0,
        k: 0,
    });
    // 1..=N: JEQ against each allowed syscall.
    for sysno in allowed {
        insns.push(libc::sock_filter {
            code: BPF_JMP | BPF_JEQ | BPF_K,
            jt: 0,
            jf: 0,
            k: *sysno as u32,
        });
    }
    // N+1: default deny.
    insns.push(libc::sock_filter {
        code: BPF_RET | BPF_K,
        jt: 0,
        jf: 0,
        k: SECCOMP_RET_KILL_PROCESS,
    });
    // N+2: allow target.
    let allow_index = insns.len();
    insns.push(libc::sock_filter {
        code: BPF_RET | BPF_K,
        jt: 0,
        jf: 0,
        k: SECCOMP_RET_ALLOW,
    });

    // Fix up each JEQ: on match jump to ALLOW, on miss fall through to the next
    // instruction (next JEQ, or the default-deny).
    for (i, _) in allowed.iter().enumerate() {
        let insn = 1 + i;
        let jt = (allow_index - insn - 1) as u8;
        insns[insn].jt = jt;
        insns[insn].jf = 0; // fall through
    }

    let prog = libc::sock_fprog {
        len: insns.len() as libc::c_ushort,
        filter: insns.as_mut_ptr(),
    };
    let rc = unsafe {
        libc::prctl(
            libc::PR_SET_SECCOMP,
            libc::SECCOMP_MODE_FILTER,
            &prog as *const libc::sock_fprog,
        )
    };
    if rc < 0 {
        return Mechanism::Unavailable(io::Error::last_os_error().to_string());
    }

    Mechanism::Engaged
}

/// A readable, one-line summary of the sandbox state, for CLI output.
pub fn report(report: &SandboxReport) -> String {
    let fmt = |m: &Mechanism| match m {
        Mechanism::Engaged => "engaged".to_string(),
        Mechanism::Unavailable(r) => format!("unavailable ({r})"),
    };
    format!(
        "seccomp: {}, landlock: {}",
        fmt(&report.seccomp),
        fmt(&report.landlock)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_is_hardened_reflects_state() {
        let fully = SandboxReport {
            seccomp: Mechanism::Engaged,
            landlock: Mechanism::Engaged,
        };
        assert!(fully.is_hardened());

        let partial = SandboxReport {
            seccomp: Mechanism::Engaged,
            landlock: Mechanism::Unavailable("nope".into()),
        };
        assert!(!partial.is_hardened());
    }

    #[test]
    fn landlock_abi_query_is_fallible_not_panicking() {
        // We cannot assert a specific ABI (host-dependent), but the query must
        // never panic and must return either Ok or a descriptive Err.
        let _ = LandlockAbi::query();
    }
}
