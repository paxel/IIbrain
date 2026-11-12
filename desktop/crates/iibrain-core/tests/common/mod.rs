//! A Brain in a temporary directory and a Device beside it, as the apps
//! would have them.

// Helpers of tests may unwrap, expect, panic and index, as tests may.
#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::fs;
use std::path::{Path, PathBuf};

use iibrain_core::Core;
use tempfile::TempDir;

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub struct World {
    pub dir: TempDir,
}

impl World {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("brain")).unwrap();
        Self { dir }
    }

    /// The Brain's folder.
    pub fn brain(&self) -> PathBuf {
        self.dir.path().join("brain")
    }

    /// The Device's own directory.
    pub fn device(&self) -> PathBuf {
        self.dir.path().join("device")
    }

    /// A second Device on the same folder.
    pub fn other_device(&self) -> PathBuf {
        self.dir.path().join("device-2")
    }

    pub fn core(&self) -> Core {
        Core::open_folder(&self.device(), &self.brain()).unwrap()
    }

    /// Writes a file into the Brain as another tool would.
    pub fn write(&self, relative: &str, content: &str) {
        let path = self.brain().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    pub fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.brain().join(relative)).unwrap()
    }

    pub fn exists(&self, relative: &str) -> bool {
        self.brain().join(relative).exists()
    }

    /// Every file of the Brain, relative, sorted.
    pub fn files(&self) -> Vec<String> {
        let mut out = Vec::new();
        walk(&self.brain(), &self.brain(), &mut out);
        out.sort();
        out
    }
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(root, &path, out);
        } else {
            out.push(
                path.strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
}

// ------------------------------------------------------- a web server

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// What a client asked the stand-in server.
#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub path: String,
    /// The header names in lower case.
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }
}

/// What the stand-in server answers.
#[derive(Debug, Clone)]
pub struct Reply {
    pub status: u16,
    pub kind: String,
    pub body: String,
    pub location: Option<String>,
}

impl Reply {
    pub fn html(body: &str) -> Self {
        Self::of(200, "text/html; charset=utf-8", body)
    }

    pub fn json(body: &str) -> Self {
        Self::of(200, "application/json", body)
    }

    pub fn status(status: u16) -> Self {
        Self::of(status, "text/plain", "")
    }

    pub fn redirect(location: &str) -> Self {
        Self {
            location: Some(location.to_string()),
            ..Self::of(302, "text/plain", "")
        }
    }

    pub fn of(status: u16, kind: &str, body: &str) -> Self {
        Self {
            status,
            kind: kind.to_string(),
            body: body.to_string(),
            location: None,
        }
    }
}

/// A web server on this machine that answers as a test tells it and
/// keeps what it was asked: a web page for a link, or a Model.
pub struct Server {
    /// `http://127.0.0.1:<port>`
    pub url: String,
    requests: Arc<Mutex<Vec<Request>>>,
    stop: Arc<AtomicBool>,
}

impl Server {
    pub fn start(answer: impl Fn(&Request) -> Reply + Send + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let (seen, stopped) = (Arc::clone(&requests), Arc::clone(&stop));
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                if stopped.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(mut stream) = stream else { continue };
                let Some(request) = read_request(&mut stream) else {
                    continue;
                };
                let reply = answer(&request);
                seen.lock().unwrap().push(request);
                let location = reply
                    .location
                    .map(|l| format!("Location: {l}\r\n"))
                    .unwrap_or_default();
                let head = format!(
                    "HTTP/1.1 {} X\r\nContent-Type: {}\r\nContent-Length: {}\r\n{location}Connection: close\r\n\r\n",
                    reply.status,
                    reply.kind,
                    reply.body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(reply.body.as_bytes());
            }
        });
        Self {
            url,
            requests,
            stop,
        }
    }

    /// Everything the server was asked so far.
    pub fn requests(&self) -> Vec<Request> {
        self.requests.lock().unwrap().clone()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // Wakes the thread out of its wait for the next client.
        let _ = TcpStream::connect(self.url.trim_start_matches("http://"));
    }
}

fn read_request(stream: &mut TcpStream) -> Option<Request> {
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 4096];
    let head_end = loop {
        let n = stream.read(&mut buffer).ok()?;
        if n == 0 {
            return None;
        }
        bytes.extend_from_slice(&buffer[..n]);
        if let Some(at) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            break at + 4;
        }
    };
    let head = String::from_utf8_lossy(&bytes[..head_end]).to_string();
    let mut lines = head.lines();
    let mut first = lines.next()?.split_whitespace();
    let method = first.next()?.to_string();
    let path = first.next()?.to_string();
    let headers: Vec<(String, String)> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(n, v)| (n.trim().to_lowercase(), v.trim().to_string()))
        .collect();
    let length: usize = headers
        .iter()
        .find(|(n, _)| n == "content-length")
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(0);
    while bytes.len() < head_end + length {
        let n = stream.read(&mut buffer).ok()?;
        if n == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..n]);
    }
    Some(Request {
        method,
        path,
        headers,
        body: String::from_utf8_lossy(&bytes[head_end..]).to_string(),
    })
}
