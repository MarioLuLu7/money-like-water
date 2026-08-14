use serde_json::{json, Value};
use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(6);

#[derive(Debug, Clone)]
pub struct CodexError {
    pub message: String,
}

impl CodexError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodexSnapshot {
    pub codex_path: String,
    pub initialize: Value,
    pub account: Value,
    pub rate_limits: Option<Value>,
    pub usage: Option<Value>,
    pub warnings: Vec<String>,
}

struct CodexRpc {
    child: Child,
    stdin: ChildStdin,
    stdout_rx: mpsc::Receiver<String>,
    stderr_lines: Arc<Mutex<Vec<String>>>,
    next_id: u64,
}

impl Drop for CodexRpc {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl CodexRpc {
    fn start(codex_path: &Path) -> Result<Self, CodexError> {
        let mut command = Command::new(codex_path);
        command
            .args(["app-server", "--stdio"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(target_os = "windows")]
        command.creation_flags(CREATE_NO_WINDOW);

        let mut child = command
            .spawn()
            .map_err(|err| CodexError::new(format!("无法启动 `codex app-server`：{err}")))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| CodexError::new("无法打开 Codex app-server stdin。"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| CodexError::new("无法打开 Codex app-server stdout。"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| CodexError::new("无法打开 Codex app-server stderr。"))?;

        let (stdout_tx, stdout_rx) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                let _ = stdout_tx.send(line);
            }
        });

        let stderr_lines = Arc::new(Mutex::new(Vec::new()));
        let stderr_sink = Arc::clone(&stderr_lines);
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if let Ok(mut lines) = stderr_sink.lock() {
                    if lines.len() < 8 {
                        lines.push(strip_ansi(&line));
                    }
                }
            }
        });

        Ok(Self {
            child,
            stdin,
            stdout_rx,
            stderr_lines,
            next_id: 1,
        })
    }

    fn request(&mut self, method: &str, params: Option<Value>) -> Result<Value, CodexError> {
        let id = self.next_id;
        self.next_id += 1;

        let mut request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
        });

        if let Some(params) = params {
            request["params"] = params;
        }

        writeln!(self.stdin, "{request}")
            .and_then(|_| self.stdin.flush())
            .map_err(|err| CodexError::new(format!("无法写入 Codex 请求 `{method}`：{err}")))?;

        loop {
            let line = self
                .stdout_rx
                .recv_timeout(REQUEST_TIMEOUT)
                .map_err(|_| CodexError::new(self.timeout_message(method)))?;

            let value: Value = serde_json::from_str(&line)
                .map_err(|err| CodexError::new(format!("Codex 返回了无效的 JSON-RPC：{err}")))?;

            if value.get("id").and_then(Value::as_u64) != Some(id) {
                continue;
            }

            if let Some(error) = value.get("error") {
                return Err(CodexError::new(format!(
                    "Codex `{method}` 调用失败：{}",
                    compact_json(error)
                )));
            }

            return Ok(value.get("result").cloned().unwrap_or(Value::Null));
        }
    }

    fn timeout_message(&self, method: &str) -> String {
        let stderr = self
            .stderr_lines
            .lock()
            .ok()
            .map(|lines| lines.join(" | "))
            .filter(|lines| !lines.is_empty())
            .unwrap_or_else(|| "没有 stderr 输出".to_string());

        format!("等待 Codex `{method}` 响应超时（{stderr}）。")
    }
}

pub fn read_snapshot() -> Result<CodexSnapshot, CodexError> {
    let codex_path = resolve_codex_cli()?;
    ensure_codex_cli(&codex_path)?;

    let mut rpc = CodexRpc::start(&codex_path)?;
    let initialize = rpc.request(
        "initialize",
        Some(json!({
            "clientInfo": {
                "name": "money-like-water",
                "title": "Money Like Water",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "capabilities": {
                "experimentalApi": true,
                "requestAttestation": false,
                "optOutNotificationMethods": [
                    "thread/started",
                    "thread/status/changed",
                    "thread/tokenUsage/updated"
                ],
            },
        })),
    )?;

    let account = rpc.request(
        "account/read",
        Some(json!({
            "refreshToken": false,
        })),
    )?;

    let mut warnings = Vec::new();
    let rate_limits = match rpc.request("account/rateLimits/read", None) {
        Ok(value) => Some(value),
        Err(err) => {
            warnings.push(err.message);
            None
        }
    };
    let usage = match rpc.request("account/usage/read", None) {
        Ok(value) => Some(value),
        Err(err) => {
            warnings.push(err.message);
            None
        }
    };

    Ok(CodexSnapshot {
        codex_path: codex_path.display().to_string(),
        initialize,
        account,
        rate_limits,
        usage,
        warnings,
    })
}

fn ensure_codex_cli(codex_path: &Path) -> Result<(), CodexError> {
    let mut command = Command::new(codex_path);
    command.arg("--version");

    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);

    let output = command
        .output()
        .map_err(|err| CodexError::new(format!("未找到 Codex CLI：{err}")))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(CodexError::new(format!(
            "`codex --version` 执行失败：{}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

fn resolve_codex_cli() -> Result<PathBuf, CodexError> {
    if let Ok(path) = env::var("CODEX_CLI_PATH") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }

    if let Some(path) = find_on_path() {
        return Ok(path);
    }

    #[cfg(target_os = "windows")]
    if let Some(path) = find_windows_extension_codex() {
        return Ok(path);
    }

    Err(CodexError::new(
        "未找到 Codex CLI：请确认终端可执行 `codex --version`，或将 codex.exe 所在目录加入系统 PATH。已检查 CODEX_CLI_PATH、PATH 和常见编辑器扩展目录。",
    ))
}

fn find_on_path() -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    let names = codex_executable_names();

    env::split_paths(&path)
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|candidate| candidate.is_file())
}

fn codex_executable_names() -> &'static [&'static str] {
    #[cfg(target_os = "windows")]
    {
        &["codex.exe", "codex.cmd", "codex.bat", "codex"]
    }

    #[cfg(not(target_os = "windows"))]
    {
        &["codex"]
    }
}

#[cfg(target_os = "windows")]
fn find_windows_extension_codex() -> Option<PathBuf> {
    extension_roots()
        .into_iter()
        .filter_map(|root| fs::read_dir(root).ok())
        .flat_map(|entries| entries.filter_map(Result::ok))
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| {
                    let name = name.to_ascii_lowercase();
                    name.starts_with("openai.chatgpt-") || name.starts_with("openai.codex-")
                })
                .unwrap_or(false)
        })
        .flat_map(extension_codex_candidates)
        .find(|candidate| candidate.is_file())
}

#[cfg(target_os = "windows")]
fn extension_roots() -> Vec<PathBuf> {
    let Some(user_profile) = env::var_os("USERPROFILE").map(PathBuf::from) else {
        return Vec::new();
    };

    [
        ".vscode\\extensions",
        ".vscode-insiders\\extensions",
        ".cursor\\extensions",
        ".windsurf\\extensions",
    ]
    .iter()
    .map(|relative| user_profile.join(relative))
    .collect()
}

#[cfg(target_os = "windows")]
fn extension_codex_candidates(extension_dir: PathBuf) -> Vec<PathBuf> {
    [
        "bin\\windows-x86_64\\codex.exe",
        "bin\\windows-aarch64\\codex.exe",
        "bin\\codex.exe",
    ]
    .iter()
    .map(|relative| extension_dir.join(relative))
    .collect()
}

fn compact_json(value: &Value) -> String {
    let raw = value.to_string();
    if raw.len() > 500 {
        format!("{}...", &raw[..500])
    } else {
        raw
    }
}

fn strip_ansi(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            output.push(ch);
        }
    }

    output
}
