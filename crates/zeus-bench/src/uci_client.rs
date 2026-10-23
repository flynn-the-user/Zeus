//! Minimal UCI client: drives a UCI engine subprocess as a GUI would.
//!
//! Protocol grounding: `Reference/protocols/UCI_Protocol.md`. This file
//! only implements the GUI-side half of that spec (the commands a GUI
//! sends and the responses it must parse), which is infrastructure for
//! *measuring* Zeus, not part of Zeus's own algorithm — it is still kept
//! strictly to the documented protocol, nothing invented.
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

// opt_type/default aren't read yet, but this is a general-purpose UCI
// option parser (used today only for UCI_Elo's min/max) that later stages
// (e.g. setting the Hash option once a transposition table exists) will
// need the rest of.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UciOption {
    pub opt_type: String,
    pub default: Option<String>,
    pub min: Option<i64>,
    pub max: Option<i64>,
}

pub struct UciEngine {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<String>,
    pub name: String,
    pub options: HashMap<String, UciOption>,
}

impl UciEngine {
    /// Spawn `cmd` and complete the `uci` handshake
    /// (Reference/protocols/UCI_Protocol.md: "uci" -> id/option* -> "uciok").
    pub fn spawn(cmd: &str, args: &[&str]) -> std::io::Result<UciEngine> {
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(l) => {
                        if tx.send(l).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let mut engine = UciEngine {
            child,
            stdin,
            rx,
            name: String::new(),
            options: HashMap::new(),
        };

        engine.send("uci")?;
        loop {
            let line = engine
                .recv_line_timeout(Duration::from_secs(10))
                .ok_or_else(|| std::io::Error::other("engine did not respond to 'uci'"))?;
            if let Some(rest) = line.strip_prefix("id name ") {
                engine.name = rest.trim().to_string();
            } else if let Some(rest) = line.strip_prefix("option ") {
                if let Some((name, opt)) = parse_option_line(rest) {
                    engine.options.insert(name, opt);
                }
            } else if line.trim() == "uciok" {
                break;
            }
        }
        Ok(engine)
    }

    pub fn send(&mut self, line: &str) -> std::io::Result<()> {
        writeln!(self.stdin, "{line}")?;
        self.stdin.flush()
    }

    pub fn recv_line_timeout(&self, timeout: Duration) -> Option<String> {
        self.rx.recv_timeout(timeout).ok()
    }

    /// `isready` -> `readyok`, per Reference/protocols/UCI_Protocol.md.
    pub fn sync(&mut self) -> std::io::Result<bool> {
        self.send("isready")?;
        loop {
            match self.recv_line_timeout(Duration::from_secs(30)) {
                Some(l) if l.trim() == "readyok" => return Ok(true),
                Some(_) => continue,
                None => return Ok(false),
            }
        }
    }

    pub fn set_option(&mut self, name: &str, value: &str) -> std::io::Result<()> {
        self.send(&format!("setoption name {name} value {value}"))
    }

    pub fn new_game(&mut self) -> std::io::Result<()> {
        self.send("ucinewgame")?;
        self.sync()?;
        Ok(())
    }

    pub fn set_position_moves(&mut self, uci_moves: &[String]) -> std::io::Result<()> {
        if uci_moves.is_empty() {
            self.send("position startpos")
        } else {
            self.send(&format!("position startpos moves {}", uci_moves.join(" ")))
        }
    }

    /// Send `go wtime .. btime .. winc .. binc ..` and wait for `bestmove`,
    /// bounded by `hard_timeout` (the GUI-/arbiter-side enforcement of the
    /// time control, independent of whether the engine manages its own
    /// clock — see zeus-bench/src/game.rs for why).
    pub fn go_and_wait_bestmove(
        &mut self,
        wtime_ms: i64,
        btime_ms: i64,
        winc_ms: i64,
        binc_ms: i64,
        hard_timeout: Duration,
    ) -> Option<String> {
        self.send(&format!(
            "go wtime {wtime_ms} btime {btime_ms} winc {winc_ms} binc {binc_ms}"
        ))
        .ok()?;
        let deadline = std::time::Instant::now() + hard_timeout;
        loop {
            let remaining = deadline.checked_duration_since(std::time::Instant::now())?;
            let line = self.recv_line_timeout(remaining)?;
            if let Some(rest) = line.strip_prefix("bestmove ") {
                return rest.split_whitespace().next().map(|s| s.to_string());
            }
        }
    }

    pub fn quit(&mut self) {
        let _ = self.send("quit");
        let _ = self.child.wait();
    }
}

impl Drop for UciEngine {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn parse_option_line(rest: &str) -> Option<(String, UciOption)> {
    // "name <id> type <t> [default <x>] [min <x>] [max <x>] [var <x>]*"
    // per Reference/protocols/UCI_Protocol.md. <id> can contain spaces, so
    // we parse up to the "type" token for the name.
    let rest = rest.strip_prefix("name ")?;
    let type_idx = rest.find(" type ")?;
    let name = rest[..type_idx].trim().to_string();
    let tail = &rest[type_idx + " type ".len()..];
    let tokens: Vec<&str> = tail.split_whitespace().collect();
    let opt_type = tokens.first()?.to_string();

    let mut default = None;
    let mut min = None;
    let mut max = None;
    let mut i = 1;
    while i < tokens.len() {
        match tokens[i] {
            "default" => {
                default = tokens.get(i + 1).map(|s| s.to_string());
                i += 2;
            }
            "min" => {
                min = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            "max" => {
                max = tokens.get(i + 1).and_then(|s| s.parse().ok());
                i += 2;
            }
            _ => i += 1,
        }
    }

    Some((
        name,
        UciOption {
            opt_type,
            default,
            min,
            max,
        },
    ))
}
