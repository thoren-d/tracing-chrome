/*!
A tracing [Layer](`ChromeLayer`) for generating a trace that can be viewed by the Chrome Trace Viewer at `chrome://tracing`.

# Usage
```no_run
use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::{registry::Registry, prelude::*};

let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
tracing_subscriber::registry().with(chrome_layer).init();

```

!*/

use tracing::{span, Event, Metadata, Subscriber};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

use json::{number::Number, object::Object, JsonValue};
use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::Sender,
        Arc, Mutex,
    },
};

use std::io::{BufWriter, Write};
use std::{
    cell::{Cell, RefCell},
    thread::JoinHandle,
};

thread_local! {
    static OUT: RefCell<Option<Sender<Message>>> = RefCell::new(None);
    static TID: RefCell<Option<u64>> = RefCell::new(None);
}

pub struct ChromeLayer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    out: Arc<Mutex<Sender<Message>>>,
    start: std::time::Instant,
    max_tid: AtomicU64,
    include_locations: bool,
    _inner: PhantomData<S>,
}

#[derive(Default)]
pub struct ChromeLayerBuilder<S>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    out_file: Option<String>,
    include_locations: bool,
    _inner: PhantomData<S>,
}

impl<S> ChromeLayerBuilder<S>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    pub fn new() -> Self {
        ChromeLayerBuilder {
            out_file: None,
            include_locations: true,
            _inner: PhantomData::default(),
        }
    }

    /**
    Set the file to which to output the trace.

    Defaults to "./trace-{unix epoch time}.json"
    */
    pub fn file(mut self, file: String) -> Self {
        self.out_file = Some(file);
        self
    }

    /**
    Include file+line with each trace entry.

    Defaults to true.

    This can add quite a bit of data to the output so turning it off might be helpful when collecting larger traces.
    */
    pub fn include_locations(mut self, include: bool) -> Self {
        self.include_locations = include;
        self
    }

    pub fn build(self) -> (ChromeLayer<S>, FlushGuard) {
        ChromeLayer::new(self)
    }
}

/// This guard will signal the thread writing the trace file to stop and join it when dropped.
pub struct FlushGuard {
    sender: Sender<Message>,
    handle: Cell<Option<JoinHandle<()>>>,
}

impl Drop for FlushGuard {
    fn drop(&mut self) {
        self.sender.send(Message::Flush).unwrap();
        self.handle.take().unwrap().join().unwrap();
    }
}

struct Callsite {
    tid: u64,
    name: &'static str,
    target: &'static str,
    file: Option<&'static str>,
    line: Option<u32>,
}

enum Message {
    Enter(f64, Callsite),
    Event(f64, Callsite),
    Exit(f64, Callsite),
    NewThread(u64, String),
    Flush,
}

impl<S> ChromeLayer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    fn new(builder: ChromeLayerBuilder<S>) -> (ChromeLayer<S>, FlushGuard) {
        let (tx, rx) = std::sync::mpsc::channel::<Message>();
        OUT.with(|val| val.replace(Some(tx.clone())));

        let out_file = builder.out_file.unwrap_or(format!(
            "./trace-{}.json",
            std::time::SystemTime::UNIX_EPOCH
                .elapsed()
                .unwrap()
                .as_secs()
        ));

        let handle = std::thread::spawn(move || {
            let write = std::fs::File::create(out_file).unwrap();
            let mut write = BufWriter::new(write);
            write.write_all(b"[").unwrap();

            for msg in rx {
                if let Message::Flush = &msg {
                    break;
                }

                let mut entry = Object::new();

                let (ph, ts, callsite) = match &msg {
                    Message::Enter(ts, callsite) => ("B", Some(ts), Some(callsite)),
                    Message::Event(ts, callsite) => ("I", Some(ts), Some(callsite)),
                    Message::Exit(ts, callsite) => ("E", Some(ts), Some(callsite)),
                    Message::NewThread(_tid, _name) => ("M", None, None),
                    Message::Flush => panic!("Was supposed to break by now."),
                };
                entry.insert("ph", ph.to_string().into());
                entry.insert("pid", 1.into());

                if let Message::NewThread(tid, name) = msg {
                    entry.insert("name", "thread_name".to_string().into());
                    entry.insert("tid", tid.into());
                    let mut args = Object::new();
                    args.insert("name", name.into());
                    entry.insert("args", args.into());
                } else {
                    let ts = ts.unwrap();
                    let callsite = callsite.unwrap();
                    entry.insert("ts", JsonValue::Number(Number::from(*ts)));
                    entry.insert("name", callsite.name.into());
                    entry.insert("cat", callsite.target.into());
                    entry.insert("tid", callsite.tid.into());

                    if let (Some(file), Some(line)) = (callsite.file, callsite.line) {
                        let mut args = Object::new();
                        args.insert("[file]", file.to_string().into());
                        args.insert("[line]", line.into());
                        entry.insert("args", args.into());
                    }
                }

                write.write_all(entry.dump().as_bytes()).unwrap();
                write.write_all(b",").unwrap();
            }
        });

        let guard = FlushGuard {
            sender: tx.clone(),
            handle: Cell::new(Some(handle)),
        };
        let layer = ChromeLayer {
            out: Arc::new(Mutex::new(tx)),
            start: std::time::Instant::now(),
            max_tid: AtomicU64::new(0),
            include_locations: builder.include_locations,
            _inner: PhantomData::default(),
        };

        (layer, guard)
    }

    fn get_tid(&self) -> (u64, bool) {
        TID.with(|value| {
            let tid = *value.borrow();
            match tid {
                Some(tid) => (tid, false),
                None => {
                    let tid = self.max_tid.fetch_add(1, Ordering::SeqCst);
                    value.replace(Some(tid));
                    (tid, true)
                }
            }
        })
    }

    fn get_callsite(&self, data: &'static Metadata) -> Callsite {
        let (tid, new_thread) = self.get_tid();
        let name = data.name();
        let target = data.target();
        let (file, line) = if self.include_locations {
            (data.file(), data.line())
        } else {
            (None, None)
        };

        if new_thread {
            let name = match std::thread::current().name() {
                Some(name) => name.to_owned(),
                None => tid.to_string(),
            };
            self.send_message(Message::NewThread(tid, name));
        }

        Callsite {
            tid,
            name,
            target,
            file,
            line,
        }
    }

    fn get_ts(&self) -> f64 {
        self.start.elapsed().as_nanos() as f64 / 1000.0
    }

    fn send_message(&self, message: Message) {
        OUT.with(move |val| {
            if val.borrow().is_some() {
                val.borrow().as_ref().unwrap().send(message).unwrap();
            } else {
                let out = self.out.lock().unwrap().clone();
                out.send(message).unwrap();
                val.replace(Some(out));
            }
        });
    }
}

impl<S> Layer<S> for ChromeLayer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span> + Send + Sync,
{
    fn on_enter(&self, id: &span::Id, ctx: Context<'_, S>) {
        let ts = self.get_ts();
        let span = ctx.span(id).expect("Span not present.");
        let callsite = self.get_callsite(span.metadata());
        self.send_message(Message::Enter(ts, callsite));
    }

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let ts = self.get_ts();
        let callsite = self.get_callsite(event.metadata());
        self.send_message(Message::Event(ts, callsite));
    }

    fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>) {
        let ts = self.get_ts();
        let span = ctx.span(id).expect("Span not present.");
        let callsite = self.get_callsite(span.metadata());
        self.send_message(Message::Exit(ts, callsite));
    }
}
