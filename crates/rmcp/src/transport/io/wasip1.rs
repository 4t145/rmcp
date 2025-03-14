use tokio::io::{AsyncRead, AsyncWrite};
use wasi::{FD_STDIN, FD_STDOUT, Fd};

pub fn stdin() -> WasiFd {
    WasiFd::stdin()
}
pub fn stdout() -> WasiFd {
    WasiFd::stdout()
}

pub struct WasiFd {
    fd: Fd,
}

impl WasiFd {
    pub fn stdin() -> Self {
        Self { fd: FD_STDIN }
    }
    pub fn stdout() -> Self {
        Self { fd: FD_STDOUT }
    }
}

impl AsyncRead for WasiFd {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut temp_buf = vec![0u8; buf.remaining()];
        unsafe {
            match wasi::fd_read(
                self.fd,
                &[wasi::Iovec {
                    buf: temp_buf.as_mut_ptr(),
                    buf_len: temp_buf.len(),
                }],
            ) {
                Ok(n) => {
                    buf.put_slice(&temp_buf[..n]);
                    std::task::Poll::Ready(Ok(()))
                }
                Err(err) => std::task::Poll::Ready(Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("WASI read error: {}", err),
                ))),
            }
        }
    }
}

impl AsyncWrite for WasiFd {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        unsafe {
            match wasi::fd_write(
                self.fd,
                &[wasi::Ciovec {
                    buf: buf.as_ptr(),
                    buf_len: buf.len(),
                }],
            ) {
                Ok(n) => std::task::Poll::Ready(Ok(n)),
                Err(err) => std::task::Poll::Ready(Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("WASI write error: {}", err),
                ))),
            }
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        self.poll_flush(cx)
    }
}
