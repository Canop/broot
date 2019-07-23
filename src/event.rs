use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Instant, Duration};
use std::sync::{mpsc::{self, Sender, Receiver, RecvError}, Arc};
use crate::task_sync::TaskLifetime;

use crossterm::{InputEvent, KeyEvent, MouseEvent, MouseButton, TerminalInput};

const DOUBLE_CLICK_MAX_DURATION: Duration = Duration::from_millis(700);

/// a valid user event
#[derive(Debug, Clone)]
pub enum Event {
    Key(KeyEvent),
    Click(u16, u16),
    DoubleClick(u16, u16),
    Wheel(i32),
}

impl Event {
    pub fn from_crossterm_event(crossterm_event: Option<InputEvent>) -> Option<Event> {
        match crossterm_event {
            Some(InputEvent::Keyboard(key)) => Some(Event::Key(key)),
            Some(InputEvent::Mouse(MouseEvent::Release(x, y))) => Some(Event::Click(x, y)),
            Some(InputEvent::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _))) => Some(Event::Wheel(-1)),
            Some(InputEvent::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _))) => Some(Event::Wheel(1)),
            _ => None,
        }
    }
}

/// an event with time of occuring
struct TimedEvent {
    time: Instant,
    event: Event,
}
impl From<Event> for TimedEvent {
    fn from(event: Event) -> Self {
        TimedEvent {
            time: Instant::now(),
            event,
        }
    }
}

/// a thread backed event listener. Can provide a task_lifetime which
/// will expire as soon as a new event is received, thus allowing
/// interruptible tasks.
pub struct EventSource {
    rx_events: Receiver<Event>,
    tx_quit: Sender<bool>,
    task_count: Arc<AtomicUsize>,
}

impl EventSource {
    /// create a new source
    pub fn new() -> EventSource {
        let (tx_events, rx_events) = mpsc::channel();
        let (tx_quit, rx_quit) = mpsc::channel();
        let task_count = Arc::new(AtomicUsize::new(0));
        let event_count = Arc::clone(&task_count);
        thread::spawn(move || {
            let input = TerminalInput::new();
            let mut last_event: Option<TimedEvent> = None;
            if let Err(e) = input.enable_mouse_mode() {
                warn!("Error while enabling mouse. {:?}", e);
            }
            let mut crossterm_events = input.read_sync();
            loop {
                let crossterm_event = crossterm_events.next();
                debug!(" => crossterm event={:?}", crossterm_event);
                if let Some(mut event) = Event::from_crossterm_event(crossterm_event) {
                    // save the event, and maybe change it
                    // (may change a click into a double-click)
                    if let Event::Click(x, y) = event {
                        if let Some(TimedEvent{time, event:Event::Click(_, last_y)}) = last_event {
                            if last_y == y && time.elapsed() < DOUBLE_CLICK_MAX_DURATION {
                                debug!("double click");
                                event = Event::DoubleClick(x, y);
                            }
                        }
                    }
                    last_event = Some(TimedEvent::from(event.clone()));
                    event_count.fetch_add(1, Ordering::SeqCst);
                    // we send the even to the receiver in the main event loop
                    tx_events.send(event).unwrap();
                    let quit = rx_quit.recv().unwrap();
                    if quit {
                        // Cleanly quitting this thread is necessary
                        //  to ensure stdin is properly closed when
                        //  we launch an external application in the same
                        //  terminal
                        // Disabling mouse mode is also necessary to let the
                        //  terminal in a proper state.
                        input.disable_mouse_mode().unwrap();
                        return;
                    }
                }
            }
        });
        EventSource {
            rx_events,
            tx_quit,
            task_count,
        }
    }

    /// either start listening again, or quit, depending on the passed bool.
    /// It's mandatory to call this with quit=true at end for a proper ending
    /// of the thread (and its resources)
    pub fn unblock(&self, quit: bool) {
        self.tx_quit.send(quit).unwrap();
    }

    /// returns a task lifetime which will end when a new event is received
    pub fn new_task_lifetime(&self) -> TaskLifetime {
        TaskLifetime::new(&self.task_count)
    }

    /// receives a new event. Blocks until there's one.
    /// Event listening will be off until the next call to unblock.
    pub fn recv(&self) -> Result<Event, RecvError> {
        self.rx_events.recv()
    }
}
