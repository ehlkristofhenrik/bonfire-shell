use command_fds::{CommandFdExt, FdMapping};
use memfd::{FileSeal, MemfdOptions};
use nix::unistd::fexecve;
use std::{
    env::{args, vars},
    error::Error,
    ffi::CString,
    io::Write,
    os::{
        fd::{AsRawFd, FromRawFd, OwnedFd},
        unix::process::CommandExt,
    },
    process::{Command, Stdio},
};

// Executable file
const SH_LEN: usize = include_bytes!("../static/bash").len();
const SH_BIN: &[u8; SH_LEN] = include_bytes!("../static/bash");
const SH_FD: i32 = 120;

// Init script
const INIT_TXT: &'static str = include_str!("../static/source.sh");
const INIT_FD: i32 = 122;

fn main() -> Result<(), Box<dyn Error>> {
    // Create memfd
    let sh_memfd = MemfdOptions::new()
        .allow_sealing(true)
        .close_on_exec(false)
        .create("tmpsh")?;

    let init_memfd = MemfdOptions::new()
        .allow_sealing(true)
        .close_on_exec(false)
        .create("tmpinit")?;

    // Get raw filedescriptor
    let mut sh_file = sh_memfd.as_file();
    let mut init_file = init_memfd.as_file();

    // Truncate file
    sh_file.set_len(SH_LEN as u64)?;
    init_file.set_len(INIT_TXT.len() as u64)?;

    // Write file content
    sh_file.write(SH_BIN)?;
    init_file.write(CString::new(INIT_TXT).unwrap_or_default().as_bytes())?;

    // Seal memory area
    // NOTE! This is to prevent execution of modified file
    sh_memfd.add_seals(&[
        FileSeal::SealGrow,
        FileSeal::SealShrink,
        FileSeal::SealWrite,
        FileSeal::SealSeal,
    ])?;
    init_memfd.add_seals(&[
        FileSeal::SealGrow,
        FileSeal::SealShrink,
        FileSeal::SealWrite,
        FileSeal::SealSeal,
    ])?;

    // Collect arguments
    let mut init_args: Vec<CString> = args()
        .into_iter()
        .map(|s| CString::new(s).unwrap_or_default())
        .collect::<Vec<CString>>();

    // Set init file to memory file
    init_args.insert(1, CString::new("--init-file").unwrap_or_default());
    init_args.insert(
        2,
        // Inject code from readonly filedescriptor using procfs
        CString::new(format!("/proc/self/fd/{}", INIT_FD)).unwrap_or_default(),
    );

    // Collect environment variables
    let init_vars: Vec<CString> = vars()
        .into_iter()
        .map(|(k, v)| CString::new(format!("{}={}", k, v)).unwrap_or_default())
        .collect();

    // Run command
    let mut cmd = unsafe {
        Command::new("")
            // Set stdin
            .stdin(Stdio::inherit())
            // Set stdout
            .stdout(Stdio::inherit())
            // Set stderr
            .stderr(Stdio::inherit())
            // Preserve fds
            .fd_mappings(vec![
                // Execuable memory file
                FdMapping {
                    parent_fd: OwnedFd::from_raw_fd(sh_file.as_raw_fd()),
                    child_fd: SH_FD,
                },
                // Source script
                FdMapping {
                    parent_fd: OwnedFd::from_raw_fd(init_file.as_raw_fd()),
                    child_fd: INIT_FD,
                },
                // Stdin
                FdMapping {
                    parent_fd: OwnedFd::from_raw_fd(0),
                    child_fd: 0,
                },
                // Stdout
                FdMapping {
                    parent_fd: OwnedFd::from_raw_fd(1),
                    child_fd: 1,
                },
                // Stderr
                FdMapping {
                    parent_fd: OwnedFd::from_raw_fd(2),
                    child_fd: 2,
                },
            ])?
            .pre_exec(move || {
                // Replace running image with executable after fork
                fexecve(SH_FD, &init_args, &init_vars).expect("Failed to run memfd executable");
                Ok(())
            })
            .spawn()?
    };

    // Wait for child to finish
    cmd.wait()?;

    Ok(())
}
