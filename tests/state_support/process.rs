//! Start the built `klar` binary on ports that are free at run time and talk to it.
use super::TempDir;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// A loopback port that was free a moment ago.
pub fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0").expect("bind port 0").local_addr().expect("local addr").port()
}

pub struct Running {
    child: Child,
    pub http: SocketAddr,
    pub wyoming: SocketAddr,
    log: PathBuf,
}

/// `root/config` and `root/data` are created; the log goes to `root/engine.log`.
pub fn command(root: &TempDir, http: SocketAddr, wyoming: SocketAddr) -> Command {
    let config = root.file("config");
    let data = root.file("data");
    std::fs::create_dir_all(&config).expect("config dir");
    std::fs::create_dir_all(&data).expect("data dir");
    let log = std::fs::File::create(root.file("engine.log")).expect("log file");
    let mut command = Command::new(env!("CARGO_BIN_EXE_klar"));
    command
        .arg("--http")
        .arg(http.to_string())
        .arg("--wyoming")
        .arg(wyoming.to_string())
        .arg("--config-dir")
        .arg(&config)
        .arg("--data-dir")
        .arg(&data)
        .env("KLAR_UI_DIR", root.file("no-ui"))
        .env("NO_COLOR", "1")
        .env_remove("KLAR_TOKEN")
        .env_remove("KLAR_SUPPORT_BUNDLE")
        .stdin(Stdio::null())
        .stdout(log.try_clone().expect("clone log"))
        .stderr(log);
    command
}

/// Start the engine and wait until both listeners accept. Retries on another port pair when
/// some other process took a port between the probe and the bind.
pub fn start(root: &TempDir, envs: &[(&str, &str)]) -> Running {
    for _ in 0..5 {
        let http = SocketAddr::from(([127, 0, 0, 1], free_port()));
        let wyoming = SocketAddr::from(([127, 0, 0, 1], free_port()));
        let mut command = command(root, http, wyoming);
        for (name, value) in envs {
            command.env(name, value);
        }
        let child = command.spawn().expect("spawn klar");
        let mut running = Running { child, http, wyoming, log: root.file("engine.log") };
        if running.wait_ready(Duration::from_secs(20)) {
            return running;
        }
    }
    panic!("klar did not come up on a free port pair");
}

impl Running {
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn log(&self) -> String {
        std::fs::read_to_string(&self.log).unwrap_or_default()
    }

    fn wait_ready(&mut self, limit: Duration) -> bool {
        let until = Instant::now() + limit;
        while Instant::now() < until {
            if self.child.try_wait().ok().flatten().is_some() {
                return false;
            }
            if TcpStream::connect(self.http).is_ok() && TcpStream::connect(self.wyoming).is_ok() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(40));
        }
        false
    }

    /// Wait for the process to end on its own. `None` when it is still running after `limit`.
    pub fn wait_exit(&mut self, limit: Duration) -> Option<ExitStatus> {
        let until = Instant::now() + limit;
        loop {
            if let Ok(Some(status)) = self.child.try_wait() {
                return Some(status);
            }
            if Instant::now() >= until {
                return None;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// Status code and body of one `GET`.
    pub fn get(&self, path: &str) -> (u16, String) {
        http_get(self.http, path).expect("http answer")
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn http_get(addr: SocketAddr, path: &str) -> std::io::Result<(u16, String)> {
    let mut stream = TcpStream::connect(addr)?;
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.write_all(format!("GET {path} HTTP/1.1\r\nHost: klar.test\r\nConnection: close\r\n\r\n").as_bytes())?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw)?;
    let text = String::from_utf8_lossy(&raw).into_owned();
    let status = text.split_whitespace().nth(1).and_then(|code| code.parse().ok()).unwrap_or(0);
    let body = text.split_once("\r\n\r\n").map(|(_, body)| body.to_string()).unwrap_or_default();
    Ok((status, body))
}
