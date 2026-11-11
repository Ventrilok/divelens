//! Link with the DiveLens Lightroom plugin (plugin side: `plugin/DiveLens.lrdevplugin/Link.lua`).
//!
//! The plugin listens on two local sockets, one per direction, and publishes
//! their ports and a one-time token in a link file in the app's settings
//! folder. The app connects to both, says hello with the token, then receives
//! the catalog name and the live photo selection. Messages are one JSON object
//! per line. The app pings every few seconds; silence means the link is lost.
//!
//! Nothing here depends on Tauri: [`run_session`] reports to a callback, the
//! Tauri side turns that into UI events.

use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Name of the link file, written by the plugin in the app's settings folder.
pub const LINK_FILE: &str = "lightroom-link.json";
/// Version of the message set; both sides must agree.
pub const PROTOCOL: u32 = 1;

/// Content of the link file.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkFile {
    pub protocol: u32,
    /// Port where the plugin receives (app → plugin).
    pub receive_port: u16,
    /// Port where the plugin sends (plugin → app).
    pub send_port: u16,
    pub token: String,
    #[serde(default)]
    pub plugin: String,
}

impl LinkFile {
    /// Reads the link file; `None` when Lightroom is not running with the
    /// plugin (no file) or the file is being written.
    pub fn read(dir: &Path) -> Option<Self> {
        let text = std::fs::read_to_string(dir.join(LINK_FILE)).ok()?;
        serde_json::from_str(&text).ok()
    }
}

/// What the UI shows about Lightroom.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub connected: bool,
    /// Catalog name, without folder or extension.
    pub catalog: Option<String>,
    pub lightroom_version: Option<String>,
    pub plugin_version: Option<String>,
    /// Number of photos selected in Lightroom (0 when nothing is selected).
    pub selected: usize,
}

/// Something the UI must react to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Status(Status),
    /// A Lightroom menu asks for this screen.
    Show(String),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum FromPlugin {
    Hello {
        protocol: u32,
        plugin: String,
        lightroom: String,
        catalog: String,
    },
    Selection {
        count: usize,
    },
    Show {
        screen: String,
    },
    Pong,
    Bye,
    Error {
        reason: String,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ToPlugin<'a> {
    Hello {
        token: &'a str,
        app: &'a str,
        protocol: u32,
    },
    Ping,
}

/// Why a session ended.
#[derive(Debug, PartialEq, Eq)]
pub enum End {
    /// Could not reach the ports in the link file (stale file, Lightroom busy).
    Unreachable,
    /// Connected but the plugin did not answer hello (wrong token, old plugin).
    NoHello(Option<String>),
    /// The plugin speaks another protocol version.
    Protocol(u32),
    /// The plugin said goodbye (Lightroom quitting, plugin disabled).
    Bye,
    /// The link went silent or closed.
    Lost,
}

/// Delays of a session; shortened in tests.
#[derive(Debug, Clone, Copy)]
pub struct Timing {
    pub connect: Duration,
    pub hello: Duration,
    pub ping_every: Duration,
    pub silence: Duration,
    pub read_poll: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(1),
            hello: Duration::from_secs(3),
            ping_every: Duration::from_secs(2),
            silence: Duration::from_secs(7),
            read_poll: Duration::from_millis(250),
        }
    }
}

fn connect(port: u16, timeout: Duration) -> Option<TcpStream> {
    let addrs = ("localhost", port).to_socket_addrs().ok()?;
    addrs
        .into_iter()
        .find_map(|addr| TcpStream::connect_timeout(&addr, timeout).ok())
}

fn send_line(stream: &mut TcpStream, message: &ToPlugin) -> std::io::Result<()> {
    let mut line = serde_json::to_vec(message).expect("message serializes");
    line.push(b'\n');
    stream.write_all(&line)
}

/// Runs one connection to the plugin until it ends; reports to `on_event`.
/// Always reports a disconnected status before returning if it had reported
/// a connected one.
pub fn run_session(
    link: &LinkFile,
    app_version: &str,
    timing: Timing,
    on_event: &mut dyn FnMut(Event),
) -> End {
    if link.protocol != PROTOCOL {
        return End::Protocol(link.protocol);
    }
    // Plugin → app first: the plugin only answers once both are connected.
    let Some(incoming) = connect(link.send_port, timing.connect) else {
        return End::Unreachable;
    };
    let Some(mut outgoing) = connect(link.receive_port, timing.connect) else {
        return End::Unreachable;
    };
    let mut status = Status::default();
    let end = session(
        link,
        app_version,
        timing,
        incoming,
        &mut outgoing,
        &mut status,
        on_event,
    );
    if status.connected {
        on_event(Event::Status(Status::default()));
    }
    end
}

fn session(
    link: &LinkFile,
    app_version: &str,
    timing: Timing,
    incoming: TcpStream,
    outgoing: &mut TcpStream,
    status: &mut Status,
    on_event: &mut dyn FnMut(Event),
) -> End {
    let hello = ToPlugin::Hello {
        token: &link.token,
        app: app_version,
        protocol: PROTOCOL,
    };
    if send_line(outgoing, &hello).is_err()
        || incoming.set_read_timeout(Some(timing.read_poll)).is_err()
    {
        return End::Unreachable;
    }
    let started = Instant::now();
    let mut last_heard = Instant::now();
    let mut last_ping = Instant::now();
    let mut refusal = None;
    let mut reader = BufReader::new(incoming);
    let mut line = Vec::new();
    loop {
        match reader.read_until(b'\n', &mut line) {
            Ok(0) => return End::Lost, // closed by the plugin
            Ok(_) if line.ends_with(b"\n") => {
                last_heard = Instant::now();
                let text = String::from_utf8_lossy(&line).trim().to_string();
                line.clear();
                if text.is_empty() {
                    continue;
                }
                let Ok(message) = serde_json::from_str::<FromPlugin>(&text) else {
                    continue;
                };
                match message {
                    FromPlugin::Hello {
                        protocol,
                        plugin,
                        lightroom,
                        catalog,
                    } => {
                        if protocol != PROTOCOL {
                            return End::Protocol(protocol);
                        }
                        status.connected = true;
                        status.catalog = Some(catalog);
                        status.lightroom_version = Some(lightroom);
                        status.plugin_version = Some(plugin);
                        on_event(Event::Status(status.clone()));
                    }
                    FromPlugin::Selection { count } if status.connected => {
                        status.selected = count;
                        on_event(Event::Status(status.clone()));
                    }
                    FromPlugin::Show { screen } if status.connected => {
                        on_event(Event::Show(screen));
                    }
                    FromPlugin::Bye => return End::Bye,
                    FromPlugin::Error { reason } => refusal = Some(reason),
                    _ => {}
                }
            }
            Ok(_) => return End::Lost, // end of stream in the middle of a line
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(_) => return End::Lost,
        }
        if !status.connected && started.elapsed() > timing.hello {
            return End::NoHello(refusal);
        }
        if last_heard.elapsed() > timing.silence {
            return End::Lost;
        }
        if status.connected && last_ping.elapsed() >= timing.ping_every {
            last_ping = Instant::now();
            if send_line(outgoing, &ToPlugin::Ping).is_err() {
                return End::Lost;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;

    fn fast() -> Timing {
        // Wide margins: CI runners (Windows especially) can be slow.
        Timing {
            connect: Duration::from_millis(1500),
            hello: Duration::from_millis(1200),
            ping_every: Duration::from_millis(150),
            silence: Duration::from_millis(1200),
            read_poll: Duration::from_millis(30),
        }
    }

    /// A stand-in for the plugin: accepts the app on both ports, then runs
    /// `script` with (lines received from the app, writer to the app).
    fn fake_plugin<F>(script: F) -> (LinkFile, thread::JoinHandle<()>)
    where
        F: FnOnce(&mut dyn Iterator<Item = String>, &mut TcpStream) + Send + 'static,
    {
        let receive = TcpListener::bind("127.0.0.1:0").unwrap();
        let send = TcpListener::bind("127.0.0.1:0").unwrap();
        let link = LinkFile {
            protocol: PROTOCOL,
            receive_port: receive.local_addr().unwrap().port(),
            send_port: send.local_addr().unwrap().port(),
            token: "secret".into(),
            plugin: "0.1.0".into(),
        };
        let handle = thread::spawn(move || {
            let (mut to_app, _) = send.accept().unwrap();
            let (from_app, _) = receive.accept().unwrap();
            let mut lines = BufReader::new(from_app).lines().map_while(Result::ok);
            script(&mut lines, &mut to_app);
        });
        (link, handle)
    }

    fn write(stream: &mut TcpStream, text: &str) {
        stream.write_all(text.as_bytes()).unwrap();
    }

    const HELLO: &str = r#"{"catalog":"Dives 2026","lightroom":"16.0","plugin":"0.1.0","protocol":1,"type":"hello"}"#;

    #[test]
    fn reads_link_file() {
        let dir = std::env::temp_dir().join(format!("divelens-link-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(LinkFile::read(&dir), None);
        std::fs::write(
            dir.join(LINK_FILE),
            r#"{"plugin":"0.1.0","protocol":1,"receivePort":51000,"sendPort":51001,"token":"abc"}"#,
        )
        .unwrap();
        let link = LinkFile::read(&dir).unwrap();
        assert_eq!((link.receive_port, link.send_port), (51000, 51001));
        assert_eq!(link.token, "abc");
        std::fs::write(dir.join(LINK_FILE), r#"{"protocol":1,"rece"#).unwrap();
        assert_eq!(LinkFile::read(&dir), None, "half-written file");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn connects_follows_selection_and_ends_on_bye() {
        let (link, plugin) = fake_plugin(|lines, to_app| {
            let hello: serde_json::Value = serde_json::from_str(&lines.next().unwrap()).unwrap();
            assert_eq!(hello["type"], "hello");
            assert_eq!(hello["token"], "secret");
            assert_eq!(hello["protocol"], 1);
            // Lightroom may add CR, blank lines, or split a line in two.
            write(to_app, &format!("{HELLO}\r\n\n"));
            write(to_app, r#"{"type":"selection","count":3,"photos":[1,2,3]}"#);
            write(to_app, "\n");
            write(
                to_app,
                "{\"type\":\"show\",\"screen\":\"tag\"}\n{\"type\":\"future\"}\n",
            );
            let ping: serde_json::Value = serde_json::from_str(&lines.next().unwrap()).unwrap();
            assert_eq!(ping["type"], "ping");
            write(to_app, "{\"type\":\"pong\"}\n{\"type\":\"bye\"}\n");
        });
        let mut events = Vec::new();
        let end = run_session(&link, "0.1.0", fast(), &mut |e| events.push(e));
        plugin.join().unwrap();
        assert_eq!(end, End::Bye);
        let connected = Status {
            connected: true,
            catalog: Some("Dives 2026".into()),
            lightroom_version: Some("16.0".into()),
            plugin_version: Some("0.1.0".into()),
            selected: 0,
        };
        assert_eq!(
            events,
            vec![
                Event::Status(connected.clone()),
                Event::Status(Status {
                    selected: 3,
                    ..connected
                }),
                Event::Show("tag".into()),
                Event::Status(Status::default()),
            ]
        );
    }

    #[test]
    fn refused_hello_ends_without_connecting() {
        let (link, plugin) = fake_plugin(|lines, to_app| {
            lines.next().unwrap();
            write(to_app, "{\"type\":\"error\",\"reason\":\"token\"}\n");
            // Messages are ignored before a valid hello.
            write(to_app, "{\"type\":\"show\",\"screen\":\"tag\"}\n");
            thread::sleep(Duration::from_millis(1800));
        });
        let mut events = Vec::new();
        let end = run_session(&link, "0.1.0", fast(), &mut |e| events.push(e));
        plugin.join().unwrap();
        assert_eq!(end, End::NoHello(Some("token".into())));
        assert!(events.is_empty());
    }

    #[test]
    fn silence_means_lost() {
        let (link, plugin) = fake_plugin(|lines, to_app| {
            lines.next().unwrap();
            write(to_app, &format!("{HELLO}\n"));
            // Lightroom hangs: never answers pings, keeps the sockets open.
            thread::sleep(Duration::from_millis(2700));
        });
        let mut events = Vec::new();
        let end = run_session(&link, "0.1.0", fast(), &mut |e| events.push(e));
        assert_eq!(end, End::Lost);
        assert_eq!(events.last(), Some(&Event::Status(Status::default())));
        plugin.join().unwrap();
    }

    #[test]
    fn closed_plugin_means_lost() {
        let (link, plugin) = fake_plugin(|lines, to_app| {
            lines.next().unwrap();
            write(to_app, &format!("{HELLO}\n"));
        });
        let mut events = Vec::new();
        let end = run_session(&link, "0.1.0", fast(), &mut |e| events.push(e));
        plugin.join().unwrap();
        assert_eq!(end, End::Lost);
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn stale_link_file_is_unreachable() {
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port(); // listener dropped: nobody on this port
        let link = LinkFile {
            protocol: PROTOCOL,
            receive_port: port,
            send_port: port,
            token: "x".into(),
            plugin: String::new(),
        };
        assert_eq!(
            run_session(&link, "0.1.0", fast(), &mut |_| {}),
            End::Unreachable
        );
    }

    #[test]
    fn other_protocol_is_refused() {
        let link = LinkFile {
            protocol: 2,
            receive_port: 1,
            send_port: 1,
            token: "x".into(),
            plugin: String::new(),
        };
        assert_eq!(
            run_session(&link, "0.1.0", fast(), &mut |_| {}),
            End::Protocol(2)
        );
    }
}
