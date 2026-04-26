//! Runtime orchestration for codex-sk.
//!
//! The runtime coordinates core engine behavior with protocol-level inputs.
//! It should not own CLI parsing, Python bindings, or protocol definitions.

use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    time::Duration,
};

use codex_sk_core::{core_info, health_check};
use codex_sk_protocol::{
    protocol_version, HealthReport, HealthStatus, ProjectError, ProjectResult,
};

const READ_TIMEOUT: Duration = Duration::from_secs(10);
const WRITE_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_REQUEST_BYTES: usize = 1024 * 1024;

/// Runtime metadata useful for diagnostics and smoke tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeInfo {
    /// Core crate package name.
    pub core_name: &'static str,
    /// Supported protocol version.
    pub protocol_version: &'static str,
}

/// Returns runtime metadata without starting any services.
#[must_use]
pub fn runtime_info() -> RuntimeInfo {
    RuntimeInfo {
        core_name: core_info().name,
        protocol_version: protocol_version(),
    }
}

/// Runs runtime health checks and returns an aggregate report.
///
/// # Errors
///
/// Returns [`ProjectError::Unhealthy`] when a required runtime component fails
/// its health check.
pub fn check_health() -> ProjectResult<HealthReport> {
    let core_report = health_check();

    if core_report.status == HealthStatus::Healthy {
        Ok(HealthReport::healthy(
            "runtime",
            format!(
                "runtime is available; {}: {}",
                core_report.component, core_report.status
            ),
        ))
    } else {
        Err(ProjectError::Unhealthy(core_report.component))
    }
}

/// Configuration for the local HTTP forwarding server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForwarderConfig {
    /// Address the forwarding server should listen on.
    pub listen_addr: SocketAddr,
    /// Local upstream address that receives forwarded requests.
    pub target_addr: SocketAddr,
}

impl ForwarderConfig {
    /// Creates a forwarding configuration from local ports.
    ///
    /// # Errors
    ///
    /// Returns [`ProjectError::InvalidConfiguration`] if either port is zero.
    pub fn local_ports(listen_port: u16, target_port: u16) -> ProjectResult<Self> {
        if listen_port == 0 {
            return Err(ProjectError::InvalidConfiguration(
                "listen port must be greater than zero".to_string(),
            ));
        }

        if target_port == 0 {
            return Err(ProjectError::InvalidConfiguration(
                "target port must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            listen_addr: SocketAddr::from(([127, 0, 0, 1], listen_port)),
            target_addr: SocketAddr::from(([127, 0, 0, 1], target_port)),
        })
    }
}

/// Starts the HTTP forwarding server and blocks until the listener fails.
///
/// # Errors
///
/// Returns [`ProjectError::Io`] if the listener cannot bind or if accepting an
/// incoming connection fails.
pub fn start_forwarding_server(config: ForwarderConfig) -> ProjectResult<()> {
    let listener = TcpListener::bind(config.listen_addr).map_err(|error| {
        ProjectError::Io(format!("failed to bind {}: {error}", config.listen_addr))
    })?;

    for connection in listener.incoming() {
        let client = connection
            .map_err(|error| ProjectError::Io(format!("failed to accept connection: {error}")))?;

        serve_client(client, config.target_addr);
    }

    Ok(())
}

/// Serves exactly one accepted connection from an existing listener.
///
/// This is primarily useful for integration-style tests that need an ephemeral
/// listening port without running an infinite server loop.
///
/// # Errors
///
/// Returns [`ProjectError::Io`] if accepting the connection fails.
pub fn serve_one_connection(listener: &TcpListener, target_addr: SocketAddr) -> ProjectResult<()> {
    let (client, _) = listener
        .accept()
        .map_err(|error| ProjectError::Io(format!("failed to accept connection: {error}")))?;

    serve_client(client, target_addr);

    Ok(())
}

fn serve_client(mut client: TcpStream, target_addr: SocketAddr) {
    if let Err(error) = forward_client_request(&mut client, target_addr) {
        let response = bad_gateway_response(&error.to_string());
        let _ = client.write_all(response.as_bytes());
        let _ = client.flush();
    }
}

fn forward_client_request(client: &mut TcpStream, target_addr: SocketAddr) -> io::Result<()> {
    configure_stream(client)?;

    let request = read_http_message(client)?;
    let mut upstream = TcpStream::connect(target_addr)?;
    configure_stream(&upstream)?;
    upstream.write_all(&request)?;
    upstream.flush()?;

    let mut response = Vec::new();
    upstream.read_to_end(&mut response)?;
    client.write_all(&response)?;
    client.flush()
}

fn configure_stream(stream: &TcpStream) -> io::Result<()> {
    stream.set_read_timeout(Some(READ_TIMEOUT))?;
    stream.set_write_timeout(Some(WRITE_TIMEOUT))
}

fn read_http_message(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut chunk = [0; 4096];

    loop {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            break;
        }

        buffer.extend_from_slice(&chunk[..read]);

        if buffer.len() > MAX_REQUEST_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "request exceeds maximum supported size",
            ));
        }

        if has_complete_request(&buffer) {
            break;
        }
    }

    if buffer.is_empty() {
        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "empty HTTP request",
        ))
    } else {
        Ok(buffer)
    }
}

fn has_complete_request(buffer: &[u8]) -> bool {
    let Some(header_end) = find_header_end(buffer) else {
        return false;
    };

    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let content_length = headers.lines().find_map(parse_content_length).unwrap_or(0);

    buffer.len() >= header_end + 4 + content_length
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn parse_content_length(line: &str) -> Option<usize> {
    let (name, value) = line.split_once(':')?;

    if name.eq_ignore_ascii_case("content-length") {
        value.trim().parse().ok()
    } else {
        None
    }
}

fn bad_gateway_response(detail: &str) -> String {
    let body = format!("Bad Gateway: {detail}\n");

    format!(
        "HTTP/1.1 502 Bad Gateway\r\ncontent-type: text/plain; charset=utf-8\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Read, thread};

    #[test]
    fn exposes_runtime_metadata() {
        let info = runtime_info();

        assert_eq!(info.core_name, "codex-sk-core");
        assert_eq!(info.protocol_version, "0.1.0");
    }

    #[test]
    fn runtime_health_check_is_healthy() {
        let report = check_health().expect("runtime health check should pass");

        assert_eq!(report.component, "runtime");
        assert!(report.is_healthy());
    }

    #[test]
    fn local_ports_rejects_zero_ports() {
        let error = ForwarderConfig::local_ports(0, 8080).expect_err("zero listen port is invalid");

        assert_eq!(
            error,
            ProjectError::InvalidConfiguration("listen port must be greater than zero".to_string())
        );
    }

    #[test]
    fn forwards_get_request_to_target() {
        let upstream = TcpListener::bind("127.0.0.1:0").expect("bind upstream");
        let upstream_addr = upstream.local_addr().expect("upstream address");
        let forwarder = TcpListener::bind("127.0.0.1:0").expect("bind forwarder");
        let forwarder_addr = forwarder.local_addr().expect("forwarder address");

        let upstream_thread = thread::spawn(move || {
            let (mut stream, _) = upstream.accept().expect("accept upstream");
            let mut request = [0; 512];
            let read = stream.read(&mut request).expect("read upstream request");
            let request = String::from_utf8_lossy(&request[..read]);
            assert!(request.starts_with("GET /health HTTP/1.1"));

            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\ncontent-length: 12\r\nconnection: close\r\n\r\nhello world\n",
                )
                .expect("write upstream response");
        });

        let forwarder_thread = thread::spawn(move || {
            serve_one_connection(&forwarder, upstream_addr).expect("serve one connection");
        });

        let mut client = TcpStream::connect(forwarder_addr).expect("connect client");
        client
            .write_all(b"GET /health HTTP/1.1\r\nhost: localhost\r\n\r\n")
            .expect("write client request");
        client
            .shutdown(std::net::Shutdown::Write)
            .expect("shutdown client write side");

        let mut response = String::new();
        client
            .read_to_string(&mut response)
            .expect("read forwarded response");

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.ends_with("hello world\n"));

        upstream_thread.join().expect("join upstream thread");
        forwarder_thread.join().expect("join forwarder thread");
    }

    #[test]
    fn returns_bad_gateway_when_target_is_unavailable() {
        let forwarder = TcpListener::bind("127.0.0.1:0").expect("bind forwarder");
        let forwarder_addr = forwarder.local_addr().expect("forwarder address");
        let unavailable_target = SocketAddr::from(([127, 0, 0, 1], unused_local_port()));

        let forwarder_thread = thread::spawn(move || {
            serve_one_connection(&forwarder, unavailable_target).expect("serve one connection");
        });

        let mut client = TcpStream::connect(forwarder_addr).expect("connect client");
        client
            .write_all(b"GET / HTTP/1.1\r\nhost: localhost\r\n\r\n")
            .expect("write client request");
        client
            .shutdown(std::net::Shutdown::Write)
            .expect("shutdown client write side");

        let mut response = String::new();
        client
            .read_to_string(&mut response)
            .expect("read error response");

        assert!(response.starts_with("HTTP/1.1 502 Bad Gateway"));
        assert!(response.contains("Bad Gateway:"));

        forwarder_thread.join().expect("join forwarder thread");
    }

    fn unused_local_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind unused port");
        listener.local_addr().expect("unused address").port()
    }
}
