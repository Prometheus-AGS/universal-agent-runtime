//! Sole owner of the inherited launch/control input; never read ahead at bootstrap.

use std::io;
#[cfg(unix)]
use std::io::Read as _;

pub struct Input {
    file: std::fs::File,
}

impl Input {
    pub fn open() -> io::Result<Self> {
        #[cfg(unix)]
        {
            use std::os::fd::BorrowedFd;
            // SAFETY: stdin is process-owned; clone creates an independently owned,
            // close-on-exec descriptor. No other sidecar code reads stdin.
            let descriptor = unsafe { BorrowedFd::borrow_raw(libc::STDIN_FILENO) }
                .try_clone_to_owned()?;
            Ok(Self { file: descriptor.into() })
        }
        #[cfg(windows)]
        {
            use std::os::windows::io::BorrowedHandle;
            use windows_sys::Win32::System::Console::{GetStdHandle, STD_INPUT_HANDLE};
            use windows_sys::Win32::Storage::FileSystem::{GetFileType, FILE_TYPE_PIPE};
            // This is a required launch contract, not an optional fallback. The
            // launcher supplies FILE_FLAG_OVERLAPPED on inherited named-pipe stdin.
            if std::env::var_os("UAR_SIDECAR_STDIN_OVERLAPPED").as_deref()
                != Some(std::ffi::OsStr::new("1")) {
                return Err(io::Error::new(io::ErrorKind::InvalidInput,
                    "Windows sidecar requires inherited overlapped named-pipe stdin"));
            }
            // SAFETY: GetStdHandle borrows the inherited input; only the clone is owned.
            let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
            if handle.is_null() || handle == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
                return Err(io::Error::last_os_error());
            }
            // SAFETY: the process-owned inherited handle is valid above. The
            // declaration cannot turn a console or regular file into a control pipe.
            if unsafe { GetFileType(handle) } != FILE_TYPE_PIPE {
                return Err(io::Error::new(io::ErrorKind::InvalidInput,
                    "Windows sidecar control input must be a pipe"));
            }
            let handle = unsafe { BorrowedHandle::borrow_raw(handle) }.try_clone_to_owned()?;
            Ok(Self { file: handle.into() })
        }
    }

    pub fn read_token_line(&mut self, limit: usize) -> io::Result<secrecy::zeroize::Zeroizing<Vec<u8>>> {
        let mut line = secrecy::zeroize::Zeroizing::new(Vec::with_capacity(limit));
        while line.len() < limit {
            let mut byte = [0];
            #[cfg(unix)]
            let count = self.file.read(&mut byte)?;
            #[cfg(windows)]
            let count = read_bootstrap_byte(&self.file, &mut byte)?;
            if count == 0 { break; }
            line.push(byte[0]);
            if byte[0] == b'\n' { break; }
        }
        Ok(line)
    }

    pub fn into_async(self) -> io::Result<AsyncInput> {
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd as _;
            let descriptor = self.file.as_raw_fd();
            // SAFETY: the descriptor remains owned by self.file through registration.
            // This launch pipe has one reader. No application code uses stdin again.
            let flags = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
            if flags == -1 || unsafe { libc::fcntl(descriptor, libc::F_SETFL, flags | libc::O_NONBLOCK) } == -1 {
                return Err(io::Error::last_os_error());
            }
            // SAFETY: the AsyncFd takes ownership of self.file, so the descriptor
            // stays open and names the same file description until it is dropped,
            // and File::as_raw_fd always returns that one descriptor.
            let input = unsafe { tokio::io::unix::AsyncFd::register(self.file) }?;
            Ok(AsyncInput(input))
        }
        #[cfg(windows)]
        {
            Ok(AsyncInput(OverlappedInput {
                file: std::sync::Arc::new(self.file),
                cancel: std::sync::Arc::new(new_event()?), pending: None,
            }))
        }
    }
}

#[cfg(unix)]
pub struct AsyncInput(tokio::io::unix::AsyncFd<std::fs::File>);

#[cfg(unix)]
impl AsyncInput {
    pub async fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        use std::os::fd::AsRawFd as _;
        loop {
            let mut readiness = self.0.readable().await?;
            match readiness.try_io(|inner| {
                // SAFETY: bytes is a live writable slice and the descriptor is owned.
                let count = unsafe { libc::read(inner.get_ref().as_raw_fd(), bytes.as_mut_ptr().cast(), bytes.len()) };
                if count < 0 { Err(io::Error::last_os_error()) } else { Ok(count as usize) }
            }) {
                Ok(result) => return result,
                Err(_) => continue, // await the next OS readiness edge, never a timer
            }
        }
    }
}

#[cfg(windows)]
pub struct AsyncInput(OverlappedInput);

#[cfg(windows)]
impl AsyncInput {
    pub async fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let pipe = &mut self.0;
        if pipe.pending.is_none() {
            let file = pipe.file.clone();
            let cancel = pipe.cancel.clone();
            let length = bytes.len();
            pipe.pending = Some(tokio::task::spawn_blocking(move || read_owned(file, cancel, length)));
        }
        let joined = pipe.pending.as_mut().expect("registered read").await;
        pipe.pending = None;
        let result = joined.map_err(|_| io::Error::other("sidecar control read worker panicked"))??;
        bytes[..result.len()].copy_from_slice(&result);
        Ok(result.len())
    }
}

impl AsyncInput {
    pub async fn close(self) -> io::Result<()> {
        #[cfg(windows)]
        {
            let mut input = self;
            let cancellation = input.0.request_cancel();
            // Even a failed cancellation request cannot certify completion.
            // Retain ownership and await the actual outstanding read outcome.
            let completion = if let Some(worker) = input.0.pending.take() {
                worker.await.map_err(|_| io::Error::other("sidecar control read worker panicked"))
                    .and_then(|result| result.map(|_| ()))
            } else { Ok(()) };
            match (cancellation, completion) {
                (Ok(()), result) => result,
                (Err(error), Ok(())) => Err(error),
                (Err(cancel), Err(join)) => Err(io::Error::other(format!("control cancellation: {cancel}; completion: {join}"))),
            }
        }
        #[cfg(unix)]
        {
            // No worker or pending kernel-owned read exists. Releasing AsyncFd
            // is synchronous after the joined control future stops waiting.
            drop(self);
            Ok(())
        }
    }
}

#[cfg(windows)]
pub struct OverlappedInput {
    file: std::sync::Arc<std::fs::File>,
    cancel: std::sync::Arc<std::os::windows::io::OwnedHandle>,
    pending: Option<tokio::task::JoinHandle<io::Result<secrecy::zeroize::Zeroizing<Vec<u8>>>>>,
}

#[cfg(windows)]
impl OverlappedInput {
    fn request_cancel(&self) -> io::Result<()> {
        use std::os::windows::io::AsRawHandle as _;
        // SAFETY: cancel is an owned manual-reset event. Signalling is idempotent.
        if unsafe { windows_sys::Win32::System::Threading::SetEvent(self.cancel.as_raw_handle()) } == 0 {
            Err(io::Error::last_os_error())
        } else { Ok(()) }
    }
}

#[cfg(windows)]
impl Drop for OverlappedInput {
    fn drop(&mut self) { let _ = self.request_cancel(); }
}

#[cfg(windows)]
fn new_event() -> io::Result<std::os::windows::io::OwnedHandle> {
    use std::os::windows::io::{FromRawHandle as _, OwnedHandle};
    // SAFETY: null security/name creates a private non-inheritable manual-reset event.
    let event = unsafe { windows_sys::Win32::System::Threading::CreateEventW(
        std::ptr::null(), 1, 0, std::ptr::null()) };
    if event.is_null() { Err(io::Error::last_os_error()) }
    else { Ok(unsafe { OwnedHandle::from_raw_handle(event) }) }
}

#[cfg(windows)]
fn read_owned(file: std::sync::Arc<std::fs::File>,
    cancel: std::sync::Arc<std::os::windows::io::OwnedHandle>, length: usize)
    -> io::Result<secrecy::zeroize::Zeroizing<Vec<u8>>> {
    use std::os::windows::io::AsRawHandle as _;
    use windows_sys::Win32::{Foundation::{ERROR_BROKEN_PIPE, ERROR_IO_PENDING, ERROR_OPERATION_ABORTED, WAIT_OBJECT_0},
        Storage::FileSystem::ReadFile,
        System::{IO::{CancelIoEx, GetOverlappedResult, OVERLAPPED}, Threading::{INFINITE, WaitForMultipleObjects}}};
    let event = new_event()?;
    let mut bytes = secrecy::zeroize::Zeroizing::new(vec![0u8; length]);
    let mut operation: OVERLAPPED = unsafe { std::mem::zeroed() };
    operation.hEvent = event.as_raw_handle();
    let mut count = 0;
    // SAFETY: the owned buffer, file, event, and OVERLAPPED remain live until the
    // operation's GetOverlappedResult acknowledgement, including cancellation.
    if unsafe { ReadFile(file.as_raw_handle(), bytes.as_mut_ptr(), length as u32,
        &mut count, &mut operation) } == 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) { bytes.clear(); return Ok(bytes); }
        if error.raw_os_error() != Some(ERROR_IO_PENDING as i32) { return Err(error); }
        let handles = [cancel.as_raw_handle(), event.as_raw_handle()];
        let event_kind = unsafe { WaitForMultipleObjects(2, handles.as_ptr(), 0, INFINITE) };
        if event_kind != WAIT_OBJECT_0 + 1 {
            // A cancellation arriving before ReadFile is still retained by the
            // manual-reset event. No CancelSynchronousIo race or repeated probe.
            unsafe { CancelIoEx(file.as_raw_handle(), &operation); }
        }
        let completed = unsafe { GetOverlappedResult(file.as_raw_handle(), &operation, &mut count, 1) };
        let completion_error = io::Error::last_os_error();
        if event_kind != WAIT_OBJECT_0 && event_kind != WAIT_OBJECT_0 + 1 {
            return Err(io::Error::other("sidecar control wait failed after read acknowledgement"));
        }
        if completed == 0 {
            let error = completion_error;
            if matches!(error.raw_os_error(), Some(code) if code == ERROR_OPERATION_ABORTED as i32 || code == ERROR_BROKEN_PIPE as i32) {
                bytes.clear(); return Ok(bytes);
            }
            return Err(error);
        }
    }
    bytes.truncate(count as usize);
    Ok(bytes)
}

#[cfg(windows)]
fn read_bootstrap_byte(file: &std::fs::File, byte: &mut [u8; 1]) -> io::Result<usize> {
    use std::os::windows::io::{AsRawHandle as _, FromRawHandle as _, OwnedHandle};
    use windows_sys::Win32::{Foundation::{ERROR_BROKEN_PIPE, ERROR_IO_PENDING},
        Storage::FileSystem::ReadFile, System::{IO::{GetOverlappedResult, OVERLAPPED}, Threading::CreateEventW}};
    // SAFETY: this private event and OVERLAPPED live until the synchronous bootstrap
    // acknowledgement. No runtime, socket, or worker exists during token ingestion.
    let raw_event = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };
    if raw_event.is_null() { return Err(io::Error::last_os_error()); }
    let event = unsafe { OwnedHandle::from_raw_handle(raw_event) };
    let mut operation: OVERLAPPED = unsafe { std::mem::zeroed() };
    operation.hEvent = event.as_raw_handle();
    let mut count = 0;
    let started = unsafe { ReadFile(file.as_raw_handle(), byte.as_mut_ptr(), 1, &mut count, &mut operation) };
    if started == 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) { return Ok(0); }
        if error.raw_os_error() != Some(ERROR_IO_PENDING as i32) { return Err(error); }
        if unsafe { GetOverlappedResult(file.as_raw_handle(), &operation, &mut count, 1) } == 0 {
            let error = io::Error::last_os_error();
            return if error.raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) { Ok(0) } else { Err(error) };
        }
    }
    Ok(count as usize)
}
