//! What the tests of the core and of the desktop app share: a Brain in
//! a temporary directory with a Device beside it, as the apps would have
//! them, and a web server on this machine that stands in for a web page
//! or a Model.

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use iibrain_core::{Core, Model};
use tempfile::TempDir;

/// What a test returns.
pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub struct World {
    pub dir: TempDir,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
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

// ------------------------------------------------------- a Model

/// A stand-in Model: an OpenAI-compatible endpoint on this machine that
/// answers every chat with what `answer` makes of the request, and
/// keeps what it was sent.
pub fn model_server(answer: impl Fn(&Request) -> String + Send + 'static) -> Server {
    Server::start(move |request| {
        let content = answer(request);
        Reply::json(
            &serde_json::json!({
                "choices": [{ "message": { "role": "assistant", "content": content } }]
            })
            .to_string(),
        )
    })
}

/// The Model at the stand-in `server`, declared local or cloud.
pub fn model(server: &Server, name: &str, local: bool) -> Model {
    Model {
        name: name.to_string(),
        address: format!("{}/v1", server.url),
        key: Some(format!("key-of-{name}")),
        model: format!("{name}-large"),
        local,
    }
}

/// Everything the stand-in Model was sent, as one text: what a test
/// searches for something that must not have left.
pub fn sent_to(server: &Server) -> String {
    server
        .requests()
        .iter()
        .map(|r| r.body.clone())
        .collect::<Vec<_>>()
        .join("\n")
}
