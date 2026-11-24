//! Start the built `klar` binary on ports that are free at run time and talk to it.
use super::TempDir;
use std::io::{BufRead, BufReader, Read, Write};
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

/// How the binary is started.
#[derive(Clone, Copy)]
pub enum Launch {
    /// As a child of the test.
    Direct,
    /// As PID 1 of a new PID namespace, the way the add-on container runs it.
    AsPidOne,
    /// With at most this many open file descriptors.
    FdLimit(u32),
}

const PID_NAMESPACE: [&str; 3] = ["unshare", "-Urpf", "--mount-proc"];

/// Whether this machine lets an unprivileged process create a PID namespace.
pub fn pid_namespace_available() -> bool {
    Command::new(PID_NAMESPACE[0]).args(&PID_NAMESPACE[1..]).arg("true").output().is_ok_and(|out| out.status.success())
}

/// `root/config` and `root/data` are created; the log goes to `root/engine.log`.
pub fn command(root: &TempDir, http: SocketAddr, wyoming: SocketAddr) -> Command {
    command_via(root, http, wyoming, Launch::Direct)
}

pub fn command_via(root: &TempDir, http: SocketAddr, wyoming: SocketAddr, launch: Launch) -> Command {
    let config = root.file("config");
    let data = root.file("data");
    std::fs::create_dir_all(&config).expect("config dir");
    std::fs::create_dir_all(&data).expect("data dir");
    let log = std::fs::OpenOptions::new().create(true).append(true).open(root.file("engine.log")).expect("log file");
    let binary = env!("CARGO_BIN_EXE_klar");
    let mut command = match launch {
        Launch::Direct => Command::new(binary),
        Launch::AsPidOne => {
            let mut command = Command::new(PID_NAMESPACE[0]);
            command.args(&PID_NAMESPACE[1..]).arg(binary);
            command
        }
        Launch::FdLimit(limit) => {
            let mut command = Command::new("sh");
            command.arg("-c").arg(format!("ulimit -n {limit} && exec \"$@\"")).arg("sh").arg(binary);
            command
        }
    };
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
        .env_remove("KLAR_LLM_MODEL")
        .stdin(Stdio::null())
        .stdout(log.try_clone().expect("clone log"))
        .stderr(log);
    command
}

/// Start the engine and wait until both listeners accept. Retries on another port pair when
/// some other process took a port between the probe and the bind.
pub fn start(root: &TempDir, envs: &[(&str, &str)]) -> Running {
    start_via(root, envs, Launch::Direct)
}

pub fn start_via(root: &TempDir, envs: &[(&str, &str)], launch: Launch) -> Running {
    for _ in 0..5 {
        let http = SocketAddr::from(([127, 0, 0, 1], free_port()));
        let wyoming = SocketAddr::from(([127, 0, 0, 1], free_port()));
        let mut running = spawn_on(root, http, wyoming, envs, launch);
        if running.wait_ready(Duration::from_secs(20)) {
            return running;
        }
    }
    panic!("klar did not come up on a free port pair");
}

/// Start the engine on exactly these addresses and do not wait for it.
pub fn spawn_on(root: &TempDir, http: SocketAddr, wyoming: SocketAddr, envs: &[(&str, &str)], launch: Launch) -> Running {
    let mut command = command_via(root, http, wyoming, launch);
    for (name, value) in envs {
        command.env(name, value);
    }
    let child = command.spawn().expect("spawn klar");
    Running { child, http, wyoming, log: root.file("engine.log") }
}

/// Process ids whose parent is `parent`.
fn children_of(parent: u32) -> Vec<u32> {
    let wanted = format!("PPid:\t{parent}");
    std::fs::read_dir("/proc")
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().to_string_lossy().parse::<u32>().ok())
        .filter(|pid| std::fs::read_to_string(format!("/proc/{pid}/status")).is_ok_and(|status| status.lines().any(|line| line == wanted)))
        .collect()
}

fn send_signal(name: &str, pid: u32) {
    let sent = Command::new("kill").arg("-s").arg(name).arg(pid.to_string()).status().expect("run kill");
    assert!(sent.success(), "kill -s {name} {pid}");
}

impl Running {
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn log(&self) -> String {
        std::fs::read_to_string(&self.log).unwrap_or_default()
    }

    /// The process id of the engine itself: the child, or under a wrapper that forks, the
    /// child of that.
    pub fn engine_pid(&self) -> u32 {
        children_of(self.child.id()).first().copied().unwrap_or_else(|| self.child.id())
    }

    /// The engine's process id inside its own PID namespace.
    pub fn pid_in_namespace(&self) -> Option<u32> {
        let status = std::fs::read_to_string(format!("/proc/{}/status", self.engine_pid())).ok()?;
        status.lines().find_map(|line| line.strip_prefix("NSpid:"))?.split_whitespace().last()?.parse().ok()
    }

    /// Send `TERM`, `INT` and the like to the engine.
    pub fn signal(&self, name: &str) {
        send_signal(name, self.engine_pid());
    }

    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
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
        // Under a wrapper that forks, killing the wrapper would leave the engine behind.
        for pid in children_of(self.child.id()) {
            let _ = Command::new("kill").arg("-s").arg("KILL").arg(pid.to_string()).status();
        }
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

/// One Wyoming `describe` exchange: the first line of the answer.
pub fn wyoming_describe(addr: SocketAddr, limit: Duration) -> std::io::Result<String> {
    let mut stream = TcpStream::connect_timeout(&addr, limit)?;
    stream.set_read_timeout(Some(limit))?;
    stream.write_all(b"{\"type\":\"describe\"}\n")?;
    let mut line = String::new();
    BufReader::new(stream).read_line(&mut line)?;
    if line.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "closed without an answer"));
    }
    Ok(line)
}
