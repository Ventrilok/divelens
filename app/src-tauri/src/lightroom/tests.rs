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
        silence_busy: Duration::from_millis(5000),
        read_poll: Duration::from_millis(30),
    }
}

/// A stand-in for the plugin: accepts the app on both ports, then runs
/// `script` with (lines received from the app, writer to the app).
fn fake_plugin<F>(script: F) -> (LinkFile, thread::JoinHandle<()>)
where
    F: FnOnce(&mut dyn Iterator<Item = String>, &mut TcpStream) + Send + 'static,
{
    // The app asks for the catalogue size after every hello: scripts
    // that are not about it never see that line.
    fake_plugin_seeing_everything(move |lines, to_app| {
        let mut lines = lines.filter(|l| !l.contains(r#""type":"countPhotos""#));
        script(&mut lines, to_app);
    })
}

/// Same, but `script` receives every line, `countPhotos` included.
fn fake_plugin_seeing_everything<F>(script: F) -> (LinkFile, thread::JoinHandle<()>)
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

/// No requests: the sending side is gone at once.
fn idle() -> Receiver<Command> {
    std::sync::mpsc::channel().1
}

fn write(stream: &mut TcpStream, text: &str) {
    stream.write_all(text.as_bytes()).unwrap();
}

const HELLO: &str =
    r#"{"catalog":"Dives 2026","lightroom":"16.0","plugin":"0.1.0","protocol":7,"type":"hello"}"#;

#[test]
fn dropped_requests_stop_their_callers_waiting() {
    let (commands, inbox) = std::sync::mpsc::channel();
    let (reply, answer) = std::sync::mpsc::channel();
    commands.send(Command::ListSets { reply }).unwrap();
    drop_requests(&inbox);
    assert!(answer.recv().is_err(), "no answer will come");
}

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
    let (link, plugin) = fake_plugin_seeing_everything(|lines, to_app| {
        let hello: serde_json::Value = serde_json::from_str(&lines.next().unwrap()).unwrap();
        assert_eq!(hello["type"], "hello");
        assert_eq!(hello["token"], "secret");
        assert_eq!(hello["protocol"], PROTOCOL);
        // Lightroom may add CR, blank lines, or split a line in two.
        write(to_app, &format!("{HELLO}\r\n\n"));
        write(
            to_app,
            r#"{"type":"selection","count":3,"version":4,"from":"2025-12-30T07:31:12","to":"2025-12-30T08:12:40"}"#,
        );
        write(to_app, "\n");
        write(
            to_app,
            "{\"type\":\"thumbnail\",\"version\":4,\"index\":1,\"jpeg\":\"/9j/\"}\n",
        );
        write(
            to_app,
            "{\"type\":\"show\",\"screen\":\"tag\"}\n{\"type\":\"future\"}\n",
        );
        // After an accepted hello: the size request, then the pings.
        let count: serde_json::Value = serde_json::from_str(&lines.next().unwrap()).unwrap();
        assert_eq!(count["type"], "countPhotos");
        let ping: serde_json::Value = serde_json::from_str(&lines.next().unwrap()).unwrap();
        assert_eq!(ping["type"], "ping");
        write(to_app, "{\"type\":\"pong\"}\n{\"type\":\"bye\"}\n");
    });
    let mut events = Vec::new();
    let end = run_session(&link, "0.1.0", fast(), &idle(), &mut |e| events.push(e));
    plugin.join().unwrap();
    assert_eq!(end, End::Bye);
    let connected = Status {
        connected: true,
        catalog: Some("Dives 2026".into()),
        lightroom_version: Some("16.0".into()),
        plugin_version: Some("0.1.0".into()),
        ..Status::default()
    };
    assert_eq!(
        events,
        vec![
            Event::Status(connected.clone()),
            Event::Status(Status {
                selected: 3,
                selection_version: 4,
                selection_from: Some("2025-12-30T07:31:12".into()),
                selection_to: Some("2025-12-30T08:12:40".into()),
                ..connected
            }),
            Event::Thumbnail {
                version: 4,
                index: 1,
                jpeg: "/9j/".into()
            },
            Event::Show("tag".into()),
            Event::Status(Status::default()),
        ]
    );
}

/// Runs a session whose plug-in is `script` (every line visible, hello
/// answered by the script); returns the events and how it ended.
fn events_of(
    script: impl FnOnce(&mut dyn Iterator<Item = String>, &mut TcpStream) + Send + 'static,
) -> (Vec<Event>, End) {
    let (link, plugin) = fake_plugin_seeing_everything(script);
    let mut events = Vec::new();
    let end = run_session(&link, "0.1.0", fast(), &idle(), &mut |e| events.push(e));
    plugin.join().unwrap();
    (events, end)
}

/// Hello, then the count request the app makes by itself; answers it
/// with `answer(id)` when given.
fn hello_and_count(
    lines: &mut dyn Iterator<Item = String>,
    to_app: &mut TcpStream,
    answer: Option<&dyn Fn(u64) -> String>,
) {
    lines.next().unwrap(); // hello
    write(to_app, &format!("{HELLO}\n"));
    let count: serde_json::Value = serde_json::from_str(&lines.next().unwrap()).unwrap();
    assert_eq!(count["type"], "countPhotos");
    if let Some(answer) = answer {
        write(to_app, &(answer(count["id"].as_u64().unwrap()) + "\n"));
    }
}

fn size_answer(count: u64) -> impl Fn(u64) -> String {
    move |id| format!(r#"{{"type":"catalogSize","id":{id},"count":{count}}}"#)
}

#[test]
fn the_catalogue_size_is_asked_after_hello() {
    let (events, end) = events_of(|lines, to_app| {
        hello_and_count(lines, to_app, Some(&size_answer(48000)));
        thread::sleep(Duration::from_millis(100));
        write(to_app, "{\"type\":\"bye\"}\n");
    });
    assert_eq!(end, End::Bye);
    let sizes: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::Status(s) => Some(s.catalog_photos),
            _ => None,
        })
        .collect();
    assert_eq!(sizes, [None, Some(48000), None]);
}

#[test]
fn the_catalogue_size_is_forgotten_when_the_link_drops() {
    let (events, _) = events_of(|lines, to_app| {
        hello_and_count(lines, to_app, Some(&size_answer(48000)));
        thread::sleep(Duration::from_millis(100));
        write(to_app, "{\"type\":\"bye\"}\n");
    });
    assert_eq!(events.last(), Some(&Event::Status(Status::default())));
    assert_eq!(Status::default().catalog_photos, None);
}

#[test]
fn an_empty_catalogue_counts_zero() {
    let (events, _) = events_of(|lines, to_app| {
        hello_and_count(lines, to_app, Some(&size_answer(0)));
        thread::sleep(Duration::from_millis(100));
        write(to_app, "{\"type\":\"bye\"}\n");
    });
    assert!(events.iter().any(|e| matches!(
        e,
        Event::Status(s) if s.connected && s.catalog_photos == Some(0)
    )));
}

#[test]
fn a_catalogue_size_nobody_asked_for_is_ignored() {
    let (events, _) = events_of(|lines, to_app| {
        hello_and_count(lines, to_app, Some(&|id| size_answer(5)(id + 1000)));
        thread::sleep(Duration::from_millis(100));
        write(to_app, "{\"type\":\"bye\"}\n");
    });
    assert!(events.iter().all(|e| match e {
        Event::Status(s) => s.catalog_photos.is_none(),
        _ => true,
    }));
}

#[test]
fn a_plugin_that_never_answers_leaves_the_size_unknown() {
    let (commands, inbox) = std::sync::mpsc::channel();
    let (outcome_tx, outcome_rx) = std::sync::mpsc::channel();
    let (link, plugin) = fake_plugin_seeing_everything(move |lines, to_app| {
        hello_and_count(lines, to_app, None);
        // The count is still unanswered: other requests are served.
        let (reply, outcome) = std::sync::mpsc::channel();
        commands
            .send(Command::Apply {
                request: Box::new(request()),
                reply,
            })
            .unwrap();
        let apply = next_request(lines);
        assert_eq!(apply["type"], "apply");
        let id = apply["id"].as_u64().unwrap();
        write(
            to_app,
            &format!("{{\"type\":\"applied\",\"id\":{id},\"count\":2}}\n"),
        );
        outcome_tx.send(outcome.recv().unwrap()).unwrap();
        write(to_app, "{\"type\":\"bye\"}\n");
    });
    let mut events = Vec::new();
    let end = run_session(&link, "0.1.0", fast(), &inbox, &mut |e| events.push(e));
    plugin.join().unwrap();
    assert_eq!(end, End::Bye);
    assert_eq!(outcome_rx.recv().unwrap().unwrap().count, 2);
    assert!(events.iter().all(|e| match e {
        Event::Status(s) => s.catalog_photos.is_none(),
        _ => true,
    }));
}

#[test]
fn a_malformed_size_is_ignored() {
    let (events, end) = events_of(|lines, to_app| {
        hello_and_count(
            lines,
            to_app,
            Some(&|id| format!(r#"{{"type":"catalogSize","id":{id},"count":"many"}}"#)),
        );
        thread::sleep(Duration::from_millis(100));
        write(to_app, "{\"type\":\"bye\"}\n");
    });
    // The session went on until the plug-in's bye.
    assert_eq!(end, End::Bye);
    assert!(events.iter().all(|e| match e {
        Event::Status(s) => s.catalog_photos.is_none(),
        _ => true,
    }));
}

#[test]
fn refreshing_the_size_does_not_wait_for_the_plugin() {
    // What `refresh_catalog_size` does: hand the request over and drop
    // the reply channel. A plug-in that never answers (pongs only) must
    // not disturb the link, and other requests are served.
    let (commands, inbox) = std::sync::mpsc::channel();
    let (outcome_tx, outcome_rx) = std::sync::mpsc::channel();
    let (link, plugin) = fake_plugin_seeing_everything(move |lines, to_app| {
        hello_and_count(lines, to_app, Some(&size_answer(10)));
        let (reply, _) = std::sync::mpsc::channel();
        commands.send(Command::CountPhotos { reply }).unwrap();
        let again = next_request(lines);
        assert_eq!(again["type"], "countPhotos");
        // Never answered: pings are, as Lightroom would.
        write(to_app, "{\"type\":\"pong\"}\n");
        let (reply, outcome) = std::sync::mpsc::channel();
        commands
            .send(Command::Apply {
                request: Box::new(request()),
                reply,
            })
            .unwrap();
        let apply = next_request(lines);
        let id = apply["id"].as_u64().unwrap();
        write(
            to_app,
            &format!("{{\"type\":\"applied\",\"id\":{id},\"count\":1}}\n"),
        );
        outcome_tx.send(outcome.recv().unwrap()).unwrap();
        write(to_app, "{\"type\":\"bye\"}\n");
    });
    let mut events = Vec::new();
    let end = run_session(&link, "0.1.0", fast(), &inbox, &mut |e| events.push(e));
    plugin.join().unwrap();
    assert_eq!(end, End::Bye);
    assert_eq!(outcome_rx.recv().unwrap().unwrap().count, 1);
}

#[test]
fn the_catalogue_size_can_be_asked_again() {
    let (commands, inbox) = std::sync::mpsc::channel();
    let (link, plugin) = fake_plugin_seeing_everything(move |lines, to_app| {
        hello_and_count(lines, to_app, Some(&size_answer(10)));
        let (reply, size) = std::sync::mpsc::channel();
        commands.send(Command::CountPhotos { reply }).unwrap();
        let again = next_request(lines);
        assert_eq!(again["type"], "countPhotos");
        let id = again["id"].as_u64().unwrap();
        write(to_app, &(size_answer(11)(id) + "\n"));
        assert_eq!(size.recv().unwrap().unwrap(), 11);
        write(to_app, "{\"type\":\"bye\"}\n");
    });
    let mut events = Vec::new();
    run_session(&link, "0.1.0", fast(), &inbox, &mut |e| events.push(e));
    plugin.join().unwrap();
    assert!(events.iter().any(|e| matches!(
        e,
        Event::Status(s) if s.catalog_photos == Some(11)
    )));
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
    let end = run_session(&link, "0.1.0", fast(), &idle(), &mut |e| events.push(e));
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
    let end = run_session(&link, "0.1.0", fast(), &idle(), &mut |e| events.push(e));
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
    let end = run_session(&link, "0.1.0", fast(), &idle(), &mut |e| events.push(e));
    plugin.join().unwrap();
    assert_eq!(end, End::Lost);
    assert_eq!(events.len(), 2);
}

fn request() -> TagRequest {
    TagRequest {
        location: Some(divelens_core::tags::LocationTags {
            country: Some("Atlantis".into()),
            city: None,
            location: Some("Coral Garden".into()),
            lat: Some(12.5),
            lon: Some(-45.25),
        }),
        ..TagRequest::default()
    }
}

/// Runs one link session in which the app asks for the collection sets;
/// returns the request's number as the plugin received it.
fn one_session_request_id() -> u64 {
    let (commands, inbox) = std::sync::mpsc::channel();
    let (id_tx, id_rx) = std::sync::mpsc::channel();
    let (link, plugin) = fake_plugin(move |lines, to_app| {
        lines.next().unwrap();
        write(to_app, &format!("{HELLO}\n"));
        lines.next().unwrap(); // first ping: the app is connected
        let (reply, sets) = std::sync::mpsc::channel();
        commands.send(Command::ListSets { reply }).unwrap();
        let id = next_request(lines)["id"].as_u64().unwrap();
        write(
            to_app,
            &format!("{{\"type\":\"sets\",\"id\":{id},\"sets\":[]}}\n"),
        );
        sets.recv().unwrap().unwrap();
        id_tx.send(id).unwrap();
        write(to_app, "{\"type\":\"bye\"}\n");
    });
    assert_eq!(
        run_session(&link, "0.1.0", fast(), &inbox, &mut |_| {}),
        End::Bye
    );
    plugin.join().unwrap();
    id_rx.recv().unwrap()
}

#[test]
fn request_numbers_never_start_again_with_a_new_link() {
    // An old selection description or photo search, numbered in a
    // previous link session, can never match a request of this one.
    let first = one_session_request_id();
    let second = one_session_request_id();
    assert!(second > first, "{second} after {first}");
}

#[test]
fn tells_when_lightroom_runs_another_version_of_the_plugin() {
    let link: LinkFile = serde_json::from_str(
        r#"{"plugin":"2.0.0","protocol":5,"receivePort":51000,"sendPort":51001,"token":"abc"}"#,
    )
    .unwrap();
    let other = Status::other_protocol(&link);
    assert!(other.restart_lightroom && !other.connected);
    assert_eq!(other.plugin_version.as_deref(), Some("2.0.0"));
    let connected = |plugin: &str| Status {
        connected: true,
        plugin_version: Some(plugin.into()),
        ..Status::default()
    };
    assert!(connected("2.0.0").other_version("2.1.0"));
    assert!(!connected("2.1.0").other_version("2.1.0"));
    assert!(!Status::default().other_version("2.1.0"));
}

#[test]
fn the_plugin_speaks_the_same_protocol() {
    let link = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugin/DiveLens.lrdevplugin/Link.lua"),
    )
    .unwrap();
    // Line by line: a Windows checkout may end lines with CRLF.
    let declared: Vec<&str> = link
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("local PROTOCOL ="))
        .collect();
    assert_eq!(
        declared,
        [format!("local PROTOCOL = {PROTOCOL}")],
        "raise PROTOCOL in Link.lua and lightroom.rs together"
    );
}

#[test]
fn describes_the_selection_then_tags_it_with_per_photo_values() {
    let (commands, inbox) = std::sync::mpsc::channel();
    let (result_tx, result_rx) = std::sync::mpsc::channel();
    let (link, plugin) = fake_plugin(move |lines, to_app| {
        lines.next().unwrap();
        write(to_app, &format!("{HELLO}\n"));
        lines.next().unwrap(); // first ping: the app is connected
        let (reply, found) = std::sync::mpsc::channel();
        commands.send(Command::DescribeSelection { reply }).unwrap();
        let describe = next_request(lines);
        assert_eq!(describe["type"], "describeSelection");
        let id = describe["id"].as_u64().unwrap();
        write(
            to_app,
            &format!(
                "{{\"type\":\"photos\",\"id\":{id},\"done\":true,\"photos\":[{{\"id\":\"11\",\"time\":\"2025-12-30T05:02:00\",\"model\":\"TG-7\"}}]}}\n"
            ),
        );
        let selection = found.recv().unwrap().unwrap();
        let point = divelens_core::profile::PhotoPoint {
            elapsed_s: 120,
            depth_m: 8.0,
            temp_c: None,
        };
        let (reply, outcome) = std::sync::mpsc::channel();
        commands
            .send(Command::Apply {
                request: Box::new(TagRequest {
                    selection: Some(selection.search),
                    photos: vec![divelens_core::tags::PhotoTags::new(
                        "11".into(),
                        Some(point),
                        240,
                        divelens_core::tags::Units::default(),
                    )],
                    ..request()
                }),
                reply,
            })
            .unwrap();
        let apply = next_request(lines);
        assert_eq!(apply["type"], "apply");
        assert_eq!(apply["selection"].as_u64(), Some(selection.search));
        assert_eq!(apply["photos"][0]["id"], "11");
        assert_eq!(apply["photos"][0]["photoDepth"], "8.0 m");
        assert_eq!(apply["photos"][0]["clockOffset"], "+04:00");
        let id = apply["id"].as_u64().unwrap();
        // The selection changed in Lightroom since: nothing written.
        write(
            to_app,
            &format!("{{\"type\":\"applyFailed\",\"id\":{id},\"reason\":\"selectionChanged\"}}\n"),
        );
        result_tx
            .send((selection, outcome.recv().unwrap()))
            .unwrap();
        write(to_app, "{\"type\":\"bye\"}\n");
    });
    let end = run_session(&link, "0.1.0", fast(), &inbox, &mut |_| {});
    plugin.join().unwrap();
    assert_eq!(end, End::Bye);
    let (selection, outcome) = result_rx.recv().unwrap();
    let outcome = outcome.unwrap_err();
    assert_eq!(selection.photos.len(), 1);
    assert_eq!(selection.photos[0].model.as_deref(), Some("TG-7"));
    assert_eq!(
        outcome,
        ApplyOutcome::Failed {
            reason: "selectionChanged".into(),
            detail: None
        }
    );
    assert!(!outcome.half_written());
}

/// Connects, waits for the app's first ping (so the app knows it is
/// connected), then queues one Apply; returns its reply channel.
fn connect_and_apply(
    lines: &mut dyn Iterator<Item = String>,
    to_app: &mut TcpStream,
    commands: &Sender<Command>,
) -> Receiver<Result<TagWritten, ApplyOutcome>> {
    lines.next().unwrap(); // hello
    write(to_app, &format!("{HELLO}\n"));
    let ping: serde_json::Value = serde_json::from_str(&lines.next().unwrap()).unwrap();
    assert_eq!(ping["type"], "ping");
    let (reply, outcome) = std::sync::mpsc::channel();
    commands
        .send(Command::Apply {
            request: Box::new(request()),
            reply,
        })
        .unwrap();
    outcome
}

/// Next message from the app that is not a ping.
fn next_request(lines: &mut dyn Iterator<Item = String>) -> serde_json::Value {
    lines
        .map(|l| serde_json::from_str::<serde_json::Value>(&l).unwrap())
        .find(|m| m["type"] != "ping")
        .unwrap()
}

#[test]
fn apply_is_answered_and_survives_a_long_write() {
    let (commands, inbox) = std::sync::mpsc::channel();
    let (outcome_tx, outcome_rx) = std::sync::mpsc::channel();
    let (link, plugin) = fake_plugin(move |lines, to_app| {
        let outcome = connect_and_apply(lines, to_app, &commands);
        let apply = next_request(lines);
        assert_eq!(apply["type"], "apply");
        assert_eq!(apply["location"]["location"], "Coral Garden");
        assert_eq!(apply["dive"], serde_json::Value::Null);
        let id = apply["id"].as_u64().unwrap();
        write(
            to_app,
            &format!("{{\"type\":\"progress\",\"id\":{id},\"done\":50,\"total\":120}}\n"),
        );
        // Lightroom writes: no answer for longer than the normal silence.
        thread::sleep(Duration::from_millis(2500));
        write(
            to_app,
            &format!("{{\"type\":\"applied\",\"id\":{id},\"count\":120}}\n"),
        );
        outcome_tx.send(outcome.recv().unwrap()).unwrap();
        write(to_app, "{\"type\":\"bye\"}\n");
    });
    let mut events = Vec::new();
    let end = run_session(&link, "0.1.0", fast(), &inbox, &mut |e| events.push(e));
    plugin.join().unwrap();
    assert_eq!(end, End::Bye);
    let outcome = outcome_rx.recv().unwrap();
    assert_eq!(
        outcome,
        Ok(TagWritten {
            count: 120,
            collection: None
        })
    );
    assert!(events.contains(&Event::Progress {
        done: 50,
        total: 120
    }));
}

#[test]
fn apply_half_written_is_a_failure() {
    let (commands, inbox) = std::sync::mpsc::channel();
    let (outcome_tx, outcome_rx) = std::sync::mpsc::channel();
    let (link, plugin) = fake_plugin(move |lines, to_app| {
        let outcome = connect_and_apply(lines, to_app, &commands);
        let id = next_request(lines)["id"].as_u64().unwrap();
        // The first write done, Lightroom refused the GPS write.
        write(
            to_app,
            &format!(
                "{{\"type\":\"applyFailed\",\"id\":{id},\"reason\":\"gpsNotWritten\",\"detail\":\"aborted\"}}\n"
            ),
        );
        outcome_tx.send(outcome.recv().unwrap()).unwrap();
    });
    run_session(&link, "0.1.0", fast(), &inbox, &mut |_| {});
    plugin.join().unwrap();
    let outcome = outcome_rx.recv().unwrap().unwrap_err();
    assert!(outcome.half_written());
    // `ApplyOutcome`'s own JSON, no longer sent to a screen: Tag Selection's
    // screen reads `ApplyError::Lightroom` (apply/errors.rs).
    assert_eq!(
        serde_json::to_value(&outcome).unwrap(),
        serde_json::json!({ "outcome": "failed", "reason": "gpsNotWritten", "detail": "aborted" })
    );
}

#[test]
fn only_a_missing_gps_write_is_half_written() {
    let failed = |reason: &str| ApplyOutcome::Failed {
        reason: reason.into(),
        detail: None,
    };
    assert!(failed("gpsNotWritten").half_written());
    for reason in ["busy", "error", "noSelection", "staleSearch", "badRequest"] {
        assert!(!failed(reason).half_written(), "{reason}");
    }
}

#[test]
fn apply_refused_by_lightroom() {
    let (commands, inbox) = std::sync::mpsc::channel();
    let (outcome_tx, outcome_rx) = std::sync::mpsc::channel();
    let (link, plugin) = fake_plugin(move |lines, to_app| {
        let outcome = connect_and_apply(lines, to_app, &commands);
        let id = next_request(lines)["id"].as_u64().unwrap();
        write(
            to_app,
            &format!("{{\"type\":\"applyFailed\",\"id\":{id},\"reason\":\"noSelection\"}}\n"),
        );
        outcome_tx.send(outcome.recv().unwrap()).unwrap();
    });
    run_session(&link, "0.1.0", fast(), &inbox, &mut |_| {});
    plugin.join().unwrap();
    assert_eq!(
        outcome_rx.recv().unwrap(),
        Err(ApplyOutcome::Failed {
            reason: "noSelection".into(),
            detail: None
        })
    );
}

#[test]
fn apply_cut_by_a_lost_link_gets_no_reply() {
    let (commands, inbox) = std::sync::mpsc::channel();
    let (outcome_tx, outcome_rx) = std::sync::mpsc::channel();
    let (link, plugin) = fake_plugin(move |lines, to_app| {
        let outcome = connect_and_apply(lines, to_app, &commands);
        next_request(lines);
        outcome_tx.send(outcome).unwrap();
        // Lightroom quits in the middle: sockets close, no answer.
    });
    let end = run_session(&link, "0.1.0", fast(), &inbox, &mut |_| {});
    plugin.join().unwrap();
    assert_eq!(end, End::Lost);
    let outcome = outcome_rx.recv().unwrap();
    assert!(outcome.recv().is_err(), "reply channel dropped");
}

#[test]
fn apply_answer_carries_the_collection_when_there_is_one() {
    let (commands, inbox) = std::sync::mpsc::channel();
    let (outcome_tx, outcome_rx) = std::sync::mpsc::channel();
    let (link, plugin) = fake_plugin(move |lines, to_app| {
        let outcome = connect_and_apply(lines, to_app, &commands);
        let apply = next_request(lines);
        let id = apply["id"].as_u64().unwrap();
        write(
            to_app,
            &format!(
                "{{\"type\":\"applied\",\"id\":{id},\"count\":5,\"collection\":{{\"collection\":\"101 - Three Arches Reef\",\"count\":5,\"created\":true}}}}\n"
            ),
        );
        outcome_tx.send(outcome.recv().unwrap()).unwrap();
        write(to_app, "{\"type\":\"bye\"}\n");
    });
    let end = run_session(&link, "0.1.0", fast(), &inbox, &mut |_| {});
    plugin.join().unwrap();
    assert_eq!(end, End::Bye);
    assert_eq!(
        outcome_rx.recv().unwrap(),
        Ok(TagWritten {
            count: 5,
            collection: Some(WrittenCollection {
                collection: Some("101 - Three Arches Reef".into()),
                count: 5,
                created: true,
            })
        })
    );
}

#[test]
fn requests_made_while_disconnected_are_never_sent() {
    let (commands, inbox) = std::sync::mpsc::channel();
    let (reply, outcome) = std::sync::mpsc::channel();
    commands
        .send(Command::Apply {
            request: Box::new(request()),
            reply,
        })
        .unwrap();
    let (link, plugin) = fake_plugin_seeing_everything(|lines, to_app| {
        lines.next().unwrap();
        write(to_app, &format!("{HELLO}\n"));
        // The size request, then only pings (the queued request was
        // dropped), then the session ends.
        let count = lines.next().unwrap();
        assert!(count.contains("countPhotos"), "unexpected: {count}");
        for line in lines.take(3) {
            assert!(line.contains("ping"), "unexpected: {line}");
        }
        write(to_app, "{\"type\":\"bye\"}\n");
    });
    let end = run_session(&link, "0.1.0", fast(), &inbox, &mut |_| {});
    plugin.join().unwrap();
    assert_eq!(end, End::Bye);
    assert!(outcome.recv().is_err());
    drop(commands);
}

#[test]
fn finds_photos_in_chunks_then_writes_collections() {
    let (commands, inbox) = std::sync::mpsc::channel();
    let (result_tx, result_rx) = std::sync::mpsc::channel();
    let (link, plugin) = fake_plugin(move |lines, to_app| {
        lines.next().unwrap();
        write(to_app, &format!("{HELLO}\n"));
        lines.next().unwrap(); // first ping: the app is connected
        let (reply, found) = std::sync::mpsc::channel();
        commands
            .send(Command::FindPhotos {
                from: "2025-12-29".into(),
                to: "2026-01-02".into(),
                reply,
            })
            .unwrap();
        let find = next_request(lines);
        assert_eq!(find["type"], "findPhotos");
        assert_eq!(
            (find["from"].as_str(), find["to"].as_str()),
            (Some("2025-12-29"), Some("2026-01-02"))
        );
        let id = find["id"].as_u64().unwrap();
        write(
            to_app,
            &format!(
                "{{\"type\":\"photos\",\"id\":{id},\"done\":false,\"photos\":[{{\"id\":\"11\",\"time\":\"2025-12-30T05:02:00\",\"model\":\"TG-7\",\"serial\":\"A1\"}}]}}\n"
            ),
        );
        write(
            to_app,
            &format!(
                "{{\"type\":\"photos\",\"id\":{id},\"done\":true,\"photos\":[{{\"id\":\"12\"}}]}}\n"
            ),
        );
        let search = found.recv().unwrap().unwrap();
        let (reply, written) = std::sync::mpsc::channel();
        commands
            .send(Command::ApplyCollections {
                request: CollectionsRequest {
                    search: search.search,
                    parent: Vec::new(),
                    group: None,
                    groups: Vec::new(),
                },
                reply,
            })
            .unwrap();
        let apply = next_request(lines);
        assert_eq!(apply["type"], "applyCollections");
        assert_eq!(apply["search"].as_u64(), Some(search.search));
        let id = apply["id"].as_u64().unwrap();
        write(
            to_app,
            &format!(
                "{{\"type\":\"collectionsApplied\",\"id\":{id},\"groups\":[{{\"collection\":\"700 - Coral Garden\",\"count\":1,\"created\":true}}]}}\n"
            ),
        );
        result_tx.send((search, written.recv().unwrap())).unwrap();
        write(to_app, "{\"type\":\"bye\"}\n");
    });
    let end = run_session(&link, "0.1.0", fast(), &inbox, &mut |_| {});
    plugin.join().unwrap();
    assert_eq!(end, End::Bye);
    let (search, written) = result_rx.recv().unwrap();
    assert_eq!(
        search.photos,
        vec![
            FoundPhoto {
                id: "11".into(),
                time: Some("2025-12-30T05:02:00".into()),
                model: Some("TG-7".into()),
                serial: Some("A1".into()),
            },
            FoundPhoto {
                id: "12".into(),
                time: None,
                model: None,
                serial: None,
            },
        ]
    );
    assert_eq!(
        written,
        Ok(vec![WrittenCollection {
            collection: Some("700 - Coral Garden".into()),
            count: 1,
            created: true,
        }])
    );
}

#[test]
fn a_group_written_without_a_collection_is_read_without_a_name() {
    // The plug-in leaves `collection` out for a group written without one
    // (Json.lua drops a nil): read as no name, and the answer still reaches
    // the app.
    let (commands, inbox) = std::sync::mpsc::channel();
    let (result_tx, result_rx) = std::sync::mpsc::channel();
    let (link, plugin) = fake_plugin(move |lines, to_app| {
        lines.next().unwrap();
        write(to_app, &format!("{HELLO}\n"));
        lines.next().unwrap(); // first ping: the app is connected
        let (reply, written) = std::sync::mpsc::channel();
        commands
            .send(Command::ApplyCollections {
                request: CollectionsRequest {
                    search: 3,
                    parent: Vec::new(),
                    group: None,
                    groups: Vec::new(),
                },
                reply,
            })
            .unwrap();
        let apply = next_request(lines);
        assert_eq!(apply["type"], "applyCollections");
        let id = apply["id"].as_u64().unwrap();
        write(
            to_app,
            &format!(
                "{{\"type\":\"collectionsApplied\",\"id\":{id},\"groups\":[{{\"count\":2,\"created\":false}},{{\"collection\":\"701 - Lantern Wall\",\"count\":1,\"created\":true}}]}}\n"
            ),
        );
        result_tx.send(written.recv().unwrap()).unwrap();
        write(to_app, "{\"type\":\"bye\"}\n");
    });
    let end = run_session(&link, "0.1.0", fast(), &inbox, &mut |_| {});
    plugin.join().unwrap();
    assert_eq!(end, End::Bye);
    assert_eq!(
        result_rx.recv().unwrap(),
        Ok(vec![
            WrittenCollection {
                collection: None,
                count: 2,
                created: false,
            },
            WrittenCollection {
                collection: Some("701 - Lantern Wall".into()),
                count: 1,
                created: true,
            },
        ])
    );
}

#[test]
fn passes_legacy_settings_log_level_and_collection_sets() {
    let (commands, inbox) = std::sync::mpsc::channel();
    let (result_tx, result_rx) = std::sync::mpsc::channel();
    let (link, plugin) = fake_plugin(move |lines, to_app| {
        lines.next().unwrap();
        write(
            to_app,
            r#"{"catalog":"C","lightroom":"16.0","plugin":"0.1.0","protocol":7,"type":"hello","legacy":{"macDivePath":"~/MacDive.sqlite","displayLogOrder":"asc"}}"#,
        );
        write(to_app, "\n");
        lines.next().unwrap(); // first ping: the app is connected
        commands
            .send(Command::SetLogLevel(LogLevel::Detailed))
            .unwrap();
        let level = next_request(lines);
        assert_eq!(
            level,
            serde_json::json!({"type": "logLevel", "level": "detailed"})
        );
        let (reply, sets) = std::sync::mpsc::channel();
        commands.send(Command::ListSets { reply }).unwrap();
        let list = next_request(lines);
        assert_eq!(list["type"], "listSets");
        let id = list["id"].as_u64().unwrap();
        write(
            to_app,
            &format!(
                "{{\"type\":\"sets\",\"id\":{id},\"sets\":[{{\"name\":\"Trips\",\"children\":[{{\"name\":\"2026\",\"children\":[]}}]}}]}}\n"
            ),
        );
        result_tx.send(sets.recv().unwrap()).unwrap();
        write(to_app, "{\"type\":\"bye\"}\n");
    });
    let mut events = Vec::new();
    let end = run_session(&link, "0.1.0", fast(), &inbox, &mut |e| events.push(e));
    plugin.join().unwrap();
    assert_eq!(end, End::Bye);
    assert_eq!(
        events[1],
        Event::Legacy(Legacy {
            mac_dive_path: Some("~/MacDive.sqlite".into()),
            display_log_order: Some("asc".into()),
        })
    );
    assert_eq!(
        result_rx.recv().unwrap(),
        Ok(vec![CollectionSet {
            name: "Trips".into(),
            children: vec![CollectionSet {
                name: "2026".into(),
                children: Vec::new(),
            }],
        }])
    );
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
        run_session(&link, "0.1.0", fast(), &idle(), &mut |_| {}),
        End::Unreachable
    );
}

#[test]
fn a_plugin_with_an_older_protocol_is_refused() {
    let (commands, inbox) = std::sync::mpsc::channel();
    let (reply, _sets) = std::sync::mpsc::channel();
    commands.send(Command::ListSets { reply }).unwrap();
    let (link, plugin) = fake_plugin_seeing_everything(|lines, to_app| {
        lines.next().unwrap();
        write(
            to_app,
            &format!("{}\n", HELLO.replace("\"protocol\":7", "\"protocol\":6")),
        );
        // Nothing more is sent to it, though a request is waiting.
        assert_eq!(lines.next(), None);
    });
    let mut events = Vec::new();
    let end = run_session(&link, "0.1.0", fast(), &inbox, &mut |e| events.push(e));
    plugin.join().unwrap();
    assert_eq!(end, End::Protocol(6));
    // Never connected (and the plug-in saw no line after its hello).
    assert!(
        events
            .iter()
            .all(|e| !matches!(e, Event::Status(s) if s.connected))
    );
}

#[test]
fn other_protocol_is_refused() {
    let link = LinkFile {
        protocol: 1,
        receive_port: 1,
        send_port: 1,
        token: "x".into(),
        plugin: String::new(),
    };
    assert_eq!(
        run_session(&link, "0.1.0", fast(), &idle(), &mut |_| {}),
        End::Protocol(1)
    );
}
