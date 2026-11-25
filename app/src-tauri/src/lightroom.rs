//! Link with the DiveLens Lightroom plugin (plugin side: `Link.lua` for the link itself, with
//! `Send.lua`, `Selection.lua`, `ApplyTags.lua` and `ApplyCollections.lua`, all in
//! `plugin/DiveLens.lrdevplugin/`).
//!
//! The plugin listens on two local sockets, one per direction, and publishes
//! their ports and a one-time token in a link file in the app's settings
//! folder. The app connects to both, says hello with the token, then receives
//! the catalog name and the live photo selection. Messages are one JSON object
//! per line. The app pings every few seconds; silence means the link is lost.
//!
//! Requests (describe and tag the selection, find the photos of a period,
//! list the collection sets, write the collections, set the log level) go
//! to the plugin through a channel of [`Command`]s; each
//! carries its own reply channel. A request made while the link is down, or
//! cut by a lost link, gets no reply: its channel is dropped, which the caller
//! sees as "link lost". Requests are never retried.
//!
//! The protocol and session in this file do not depend on Tauri:
//! [`run_session`] reports to a callback. The Tauri side lives in the
//! submodules: [`events`] turns that callback into UI events, [`commands`]
//! holds what the screens invoke.

pub(crate) mod commands;
pub(crate) mod events;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use divelens_core::tags::{CollectionsRequest, TagRequest};
use serde::{Deserialize, Serialize};

use crate::settings::{Legacy, LogLevel};

/// Name of the link file, written by the plugin in the app's settings folder.
pub const LINK_FILE: &str = "lightroom-link.json";
/// Version of the message set; both sides must agree.
pub const PROTOCOL: u32 = 7;

/// Every reason the plug-in gives in an `applyFailed` message.
pub const APPLY_REASONS: &[&str] = &[
    "error",
    "busy",
    "badRequest",
    "gpsNotWritten",
    "noSelection",
    "selectionChanged",
    "staleSelection",
    "staleSearch",
    "parentNotFound",
];

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
    /// Number of photos in the whole catalog; `None` until the plugin has
    /// answered (and whenever the link is down).
    pub catalog_photos: Option<u64>,
    /// Changes with every new selection; thumbnails carry it.
    pub selection_version: u64,
    /// Earliest and latest capture times of the selection, wall-clock
    /// `YYYY-MM-DDTHH:MM:SS` as Lightroom shows them.
    pub selection_from: Option<String>,
    pub selection_to: Option<String>,
    /// Same, per camera of the selection (the remembered clock offsets apply
    /// to these).
    pub selection_cameras: Vec<CameraRange>,
    /// The selection's range moved by each camera's remembered clock offset
    /// (Tag Selection's "covers the selection" hint); filled by the app.
    pub cover_from: Option<String>,
    pub cover_to: Option<String>,
    /// Lightroom runs a Dive Lens plug-in of another version than this app,
    /// or one speaking another protocol (then not connected): Dive Lens was
    /// updated and Lightroom must restart to load its new plug-in (spec
    /// F13). Filled by the app.
    pub restart_lightroom: bool,
}

impl Status {
    /// What the UI shows while Lightroom runs a plug-in speaking another
    /// protocol than this app: not connected, Lightroom to restart.
    pub fn other_protocol(link: &LinkFile) -> Self {
        Self {
            plugin_version: (!link.plugin.is_empty()).then(|| link.plugin.clone()),
            restart_lightroom: true,
            ..Self::default()
        }
    }

    /// Whether the plug-in connected is another version than the app's.
    pub fn other_version(&self, app_version: &str) -> bool {
        self.connected
            && self
                .plugin_version
                .as_deref()
                .is_some_and(|plugin| plugin != app_version)
    }
}

/// Capture times of the selected photos of one camera.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraRange {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub serial: Option<String>,
    pub from: String,
    pub to: String,
}

/// One photo of a period, as the plugin describes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FoundPhoto {
    /// Lightroom's identifier, valid for this link session.
    pub id: String,
    /// Original capture time, wall-clock `YYYY-MM-DDTHH:MM:SS`.
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub serial: Option<String>,
}

/// Photos of a period; `search` identifies the search for the write that
/// follows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoSearch {
    pub search: u64,
    pub photos: Vec<FoundPhoto>,
}

/// What "Tag Selection" answers once the plugin applied it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagWritten {
    /// The number of photos written.
    pub count: usize,
    /// The collection the selection went in, when the request asked for one.
    pub collection: Option<WrittenCollection>,
}

/// One collection written by "Create Collections" or "Tag Selection".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WrittenCollection {
    /// None for a group written without a collection.
    pub collection: Option<String>,
    pub count: usize,
    /// The collection did not exist before.
    pub created: bool,
}

/// A collection set of the catalog and the sets inside it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionSet {
    pub name: String,
    #[serde(default)]
    pub children: Vec<CollectionSet>,
}

/// Something the UI must react to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Status(Status),
    /// The previous plugin's settings, sent once per connection when
    /// Lightroom still has them.
    Legacy(Legacy),
    /// A Lightroom menu asks for this screen.
    Show(String),
    /// Progress of the catalog write in course.
    Progress {
        done: usize,
        total: usize,
    },
    /// One thumbnail of the selection (base64 JPEG), `index` from 1.
    Thumbnail {
        version: u64,
        index: usize,
        jpeg: String,
    },
}

/// A request for the plugin.
pub enum Command {
    /// Tag the photos selected in Lightroom at the time the plugin gets it.
    Apply {
        request: Box<TagRequest>,
        /// What was written, or Lightroom's refusal or failure (including
        /// `gpsNotWritten`: written, but for the GPS and altitude).
        reply: Sender<Result<TagWritten, ApplyOutcome>>,
    },
    /// Every photo of the catalog captured between two days (`YYYY-MM-DD`,
    /// inclusive).
    FindPhotos {
        from: String,
        to: String,
        reply: Sender<Result<PhotoSearch, ApplyOutcome>>,
    },
    /// The photos selected in Lightroom now (Tag Selection's per-photo
    /// values). The plugin keeps them: an "Apply" naming this search writes
    /// only if the selection is still exactly these photos.
    DescribeSelection {
        reply: Sender<Result<PhotoSearch, ApplyOutcome>>,
    },
    /// Write collections, dive data and per-photo values in one Undo step.
    ApplyCollections {
        request: CollectionsRequest,
        reply: Sender<Result<Vec<WrittenCollection>, ApplyOutcome>>,
    },
    /// The catalog's collection sets, as a tree.
    ListSets {
        reply: Sender<Result<Vec<CollectionSet>, ApplyOutcome>>,
    },
    /// The number of photos in the whole catalog; the answer also lands in
    /// the status (`catalog_photos`).
    CountPhotos {
        reply: Sender<Result<u64, ApplyOutcome>>,
    },
    /// What the plugin writes in its log from now on; no answer.
    SetLogLevel(LogLevel),
}

/// A request waiting for the plugin's answer.
enum Pending {
    Apply(Sender<Result<TagWritten, ApplyOutcome>>),
    Find(Sender<Result<PhotoSearch, ApplyOutcome>>, Vec<FoundPhoto>),
    Collections(Sender<Result<Vec<WrittenCollection>, ApplyOutcome>>),
    Sets(Sender<Result<Vec<CollectionSet>, ApplyOutcome>>),
    /// A catalog size request; `None` when the app asked by itself.
    Count(Option<Sender<Result<u64, ApplyOutcome>>>),
}

impl Pending {
    fn fail(self, outcome: ApplyOutcome) {
        // A caller that stopped waiting is no longer interested.
        match self {
            Pending::Apply(reply) => drop(reply.send(Err(outcome))),
            Pending::Find(reply, _) => drop(reply.send(Err(outcome))),
            Pending::Collections(reply) => drop(reply.send(Err(outcome))),
            Pending::Sets(reply) => drop(reply.send(Err(outcome))),
            Pending::Count(reply) => {
                if let Some(reply) = reply {
                    drop(reply.send(Err(outcome)));
                }
            }
        }
    }
}

/// How an "Apply" ended in Lightroom.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "outcome", rename_all = "camelCase")]
pub enum ApplyOutcome {
    /// Nothing written. `reason`: `noSelection`, `busy` (Lightroom kept the
    /// catalog busy), `badRequest`, `staleSearch` (the photos were found in
    /// an earlier link session), `parentNotFound` (the collection set chosen
    /// in the settings is not in the catalog) or `error`. Except
    /// `gpsNotWritten`: everything but the GPS and altitude was written (the
    /// second catalog write did not happen).
    Failed {
        reason: String,
        detail: Option<String>,
    },
}

impl ApplyOutcome {
    /// Failed after the first catalog write: the photos carry everything
    /// but the GPS and altitude (dive data, location, per-photo values,
    /// clock offsets, collections).
    pub fn half_written(&self) -> bool {
        matches!(self, Self::Failed { reason, .. } if reason == "gpsNotWritten")
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum FromPlugin {
    Hello {
        protocol: u32,
        plugin: String,
        lightroom: String,
        catalog: String,
        #[serde(default)]
        legacy: Option<Legacy>,
    },
    Selection {
        count: usize,
        #[serde(default)]
        version: u64,
        #[serde(default)]
        from: Option<String>,
        #[serde(default)]
        to: Option<String>,
        #[serde(default)]
        cameras: Vec<CameraRange>,
    },
    Thumbnail {
        version: u64,
        index: usize,
        jpeg: String,
    },
    Show {
        screen: String,
    },
    Pong,
    Bye,
    Error {
        reason: String,
    },
    Progress {
        done: usize,
        total: usize,
    },
    Applied {
        id: u64,
        count: usize,
        #[serde(default)]
        collection: Option<WrittenCollection>,
    },
    ApplyFailed {
        id: u64,
        reason: String,
        #[serde(default)]
        detail: Option<String>,
    },
    Photos {
        id: u64,
        photos: Vec<FoundPhoto>,
        done: bool,
    },
    CollectionsApplied {
        id: u64,
        groups: Vec<WrittenCollection>,
    },
    Sets {
        id: u64,
        sets: Vec<CollectionSet>,
    },
    CatalogSize {
        id: u64,
        count: u64,
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
    Apply {
        id: u64,
        #[serde(flatten)]
        request: &'a TagRequest,
    },
    FindPhotos {
        id: u64,
        from: &'a str,
        to: &'a str,
    },
    DescribeSelection {
        id: u64,
    },
    ApplyCollections {
        id: u64,
        #[serde(flatten)]
        request: &'a CollectionsRequest,
    },
    ListSets {
        id: u64,
    },
    CountPhotos {
        id: u64,
    },
    LogLevel {
        level: LogLevel,
    },
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
    /// Silence tolerated while Lightroom writes (it cannot answer meanwhile).
    pub silence_busy: Duration,
    pub read_poll: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(1),
            hello: Duration::from_secs(3),
            ping_every: Duration::from_secs(2),
            silence: Duration::from_secs(7),
            silence_busy: Duration::from_secs(120),
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

/// Number of the next request to the plugin. It never starts again while
/// the app runs, not even when the link does: a photo search or a selection
/// description the plugin kept from an earlier link session can never be
/// taken for a newer one. (The plugin forgets them when the link resets,
/// and the app forgets its own when it quits.)
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// Drops the requests waiting to be sent: whoever made one stops waiting
/// for its answer (it gets none).
pub fn drop_requests(commands: &Receiver<Command>) {
    while commands.try_recv().is_ok() {}
}

/// Runs one connection to the plugin until it ends; reports to `on_event`.
/// Always reports a disconnected status before returning if it had reported
/// a connected one.
pub fn run_session(
    link: &LinkFile,
    app_version: &str,
    timing: Timing,
    commands: &Receiver<Command>,
    on_event: &mut dyn FnMut(Event),
) -> End {
    // Requests made while the link was down are dropped, never sent late.
    drop_requests(commands);
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
    let hello = ToPlugin::Hello {
        token: &link.token,
        app: app_version,
        protocol: PROTOCOL,
    };
    if send_line(&mut outgoing, &hello).is_err() {
        return End::Unreachable;
    }
    let mut status = Status::default();
    let end = session(
        timing,
        incoming,
        &mut outgoing,
        &mut status,
        commands,
        on_event,
    );
    if status.connected {
        on_event(Event::Status(Status::default()));
    }
    end
}

fn session(
    timing: Timing,
    incoming: TcpStream,
    outgoing: &mut TcpStream,
    status: &mut Status,
    commands: &Receiver<Command>,
    on_event: &mut dyn FnMut(Event),
) -> End {
    if incoming.set_read_timeout(Some(timing.read_poll)).is_err() {
        return End::Unreachable;
    }
    let started = Instant::now();
    let mut last_heard = Instant::now();
    let mut last_ping = Instant::now();
    let mut refusal = None;
    let mut reader = BufReader::new(incoming);
    let mut line = Vec::new();
    let mut pending: HashMap<u64, Pending> = HashMap::new();
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
                        legacy,
                    } => {
                        if protocol != PROTOCOL {
                            return End::Protocol(protocol);
                        }
                        status.connected = true;
                        status.catalog = Some(catalog);
                        status.lightroom_version = Some(lightroom);
                        status.plugin_version = Some(plugin);
                        on_event(Event::Status(status.clone()));
                        if let Some(legacy) = legacy {
                            on_event(Event::Legacy(legacy));
                        }
                        // The catalog size is asked for at once; nothing
                        // waits for the answer.
                        let id = next_id();
                        if send_line(outgoing, &ToPlugin::CountPhotos { id }).is_err() {
                            return End::Lost;
                        }
                        pending.insert(id, Pending::Count(None));
                    }
                    FromPlugin::Selection {
                        count,
                        version,
                        from,
                        to,
                        cameras,
                    } if status.connected => {
                        status.selected = count;
                        status.selection_version = version;
                        status.selection_from = from;
                        status.selection_to = to;
                        status.selection_cameras = cameras;
                        on_event(Event::Status(status.clone()));
                    }
                    FromPlugin::Thumbnail {
                        version,
                        index,
                        jpeg,
                    } if status.connected => {
                        on_event(Event::Thumbnail {
                            version,
                            index,
                            jpeg,
                        });
                    }
                    FromPlugin::Show { screen } if status.connected => {
                        on_event(Event::Show(screen));
                    }
                    FromPlugin::Progress { done, total } => {
                        on_event(Event::Progress { done, total });
                    }
                    FromPlugin::Applied {
                        id,
                        count,
                        collection,
                    } => match pending.remove(&id) {
                        Some(Pending::Apply(reply)) => {
                            let _ = reply.send(Ok(TagWritten { count, collection }));
                        }
                        Some(other) => other.fail(ApplyOutcome::Failed {
                            reason: "error".into(),
                            detail: Some("unexpected answer".into()),
                        }),
                        None => {}
                    },
                    FromPlugin::ApplyFailed { id, reason, detail } => {
                        if let Some(p) = pending.remove(&id) {
                            p.fail(ApplyOutcome::Failed { reason, detail });
                        }
                    }
                    FromPlugin::Photos { id, photos, done } => {
                        if let Some(Pending::Find(_, found)) = pending.get_mut(&id) {
                            found.extend(photos);
                        }
                        if done && let Some(Pending::Find(reply, found)) = pending.remove(&id) {
                            let _ = reply.send(Ok(PhotoSearch {
                                search: id,
                                photos: found,
                            }));
                        }
                    }
                    FromPlugin::CollectionsApplied { id, groups } => {
                        if let Some(Pending::Collections(reply)) = pending.remove(&id) {
                            let _ = reply.send(Ok(groups));
                        }
                    }
                    FromPlugin::Sets { id, sets } => {
                        if let Some(Pending::Sets(reply)) = pending.remove(&id) {
                            let _ = reply.send(Ok(sets));
                        }
                    }
                    FromPlugin::CatalogSize { id, count } => {
                        // An answer nobody asked for (unknown id) is ignored.
                        if let Some(Pending::Count(reply)) = pending.remove(&id) {
                            status.catalog_photos = Some(count);
                            on_event(Event::Status(status.clone()));
                            if let Some(reply) = reply {
                                let _ = reply.send(Ok(count));
                            }
                        }
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
        // The catalog count is quick and nobody waits on it: it never
        // stretches the silence the link puts up with.
        let busy = pending.values().any(|p| !matches!(p, Pending::Count(_)));
        let silence = if !busy {
            timing.silence
        } else {
            timing.silence_busy
        };
        if last_heard.elapsed() > silence {
            return End::Lost;
        }
        while let Ok(command) = commands.try_recv() {
            if !status.connected {
                continue; // dropped: the caller learns the link is not up
            }
            let id = next_id();
            let message = match &command {
                Command::Apply { request, .. } => ToPlugin::Apply {
                    id,
                    request: request.as_ref(),
                },
                Command::FindPhotos { from, to, .. } => ToPlugin::FindPhotos { id, from, to },
                Command::DescribeSelection { .. } => ToPlugin::DescribeSelection { id },
                Command::ApplyCollections { request, .. } => {
                    ToPlugin::ApplyCollections { id, request }
                }
                Command::ListSets { .. } => ToPlugin::ListSets { id },
                Command::CountPhotos { .. } => ToPlugin::CountPhotos { id },
                Command::SetLogLevel(level) => ToPlugin::LogLevel { level: *level },
            };
            if send_line(outgoing, &message).is_err() {
                return End::Lost;
            }
            let waiting = match command {
                Command::Apply { reply, .. } => Pending::Apply(reply),
                Command::FindPhotos { reply, .. } | Command::DescribeSelection { reply } => {
                    Pending::Find(reply, Vec::new())
                }
                Command::ApplyCollections { reply, .. } => Pending::Collections(reply),
                Command::ListSets { reply } => Pending::Sets(reply),
                Command::CountPhotos { reply } => Pending::Count(Some(reply)),
                Command::SetLogLevel(_) => continue,
            };
            pending.insert(id, waiting);
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
mod tests;
