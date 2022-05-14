use lazy_static::lazy_static;
use libspnav_bindings as libspnav;
use std::convert::{From, Into, TryFrom};
use std::sync::Mutex;

#[derive(Debug, Clone, Copy)]
pub enum EventType {
    Any,
    Motion,
    Button,
}

const SPNAV_EVENT_ANY: i32 = 0;
const SPNAV_EVENT_MOTION: i32 = 1;
const SPNAV_EVENT_BUTTON: i32 = 2;

impl Into<i32> for EventType {
    fn into(self) -> i32 {
        match self {
            EventType::Any => SPNAV_EVENT_ANY,
            EventType::Motion => SPNAV_EVENT_MOTION,
            EventType::Button => SPNAV_EVENT_BUTTON,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Motion(MotionEvent),
    Button(ButtonEvent),
}

#[derive(Debug, Clone)]
pub struct MotionEvent {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub rx: i32,
    pub ry: i32,
    pub rz: i32,
    pub period: u32,
    // data[6] not included, because I'm not sure if its redundant
}

impl MotionEvent {
    // Convenience method that returns x, y, z translation
    pub fn t(&self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }
    // Convenience method that returns  rx, ry, rz rotation
    pub fn r(&self) -> (i32, i32, i32) {
        (self.rx, self.ry, self.rz)
    }
}

impl From<libspnav::spnav_event_motion> for MotionEvent {
    fn from(event: libspnav::spnav_event_motion) -> Self {
        MotionEvent {
            x: event.x,
            y: event.y,
            z: event.z,
            rx: event.rx,
            ry: event.ry,
            rz: event.rz,
            period: event.period,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ButtonEvent {
    pub press: bool,
    pub bnum: i32,
}

impl From<libspnav::spnav_event_button> for ButtonEvent {
    fn from(event: libspnav::spnav_event_button) -> Self {
        ButtonEvent {
            press: event.press != 0,
            bnum: event.bnum,
        }
    }
}

impl TryFrom<libspnav::spnav_event> for Event {
    type Error = ();
    fn try_from(event: libspnav::spnav_event) -> Result<Self, Self::Error> {
        unsafe {
            match event {
                libspnav::spnav_event {
                    type_: SPNAV_EVENT_MOTION,
                } => Ok(Event::Motion(event.motion.into())),
                libspnav::spnav_event {
                    type_: SPNAV_EVENT_BUTTON,
                } => Ok(Event::Button(event.button.into())),
                _ => Err(()),
            }
        }
    }
}

#[derive(Debug)]
pub struct Connection {
    pub fd: i32,
}

lazy_static! {
    static ref CONN_COUNT: Mutex<usize> = Mutex::new(0);
}

impl Connection {
    pub fn new() -> Result<Connection, ()> {
        let mut count = CONN_COUNT.lock().expect("to lock");
        if *count > 0 {
            *count += 1;
            Ok(Connection {
                fd: lib::spnav_fd()?,
            })
        } else {
            *count = 1;
            lib::spnav_open()?;
            Ok(Connection {
                fd: lib::spnav_fd()?,
            })
        }
    }
    pub fn poll(&self) -> Option<Event> {
        lib::spnav_poll_event()
    }
    pub fn wait(&self) -> Result<Event, ()> {
        lib::spnav_wait_event()
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let mut count = CONN_COUNT.lock().expect("to lock");
        if *count == 1 {
            *count = 0;
            lib::spnav_close().expect("to close");
        } else {
            *count -= 1;
        }
    }
}

pub mod lib {
    use super::*;

    /* Open connection to the daemon via AF_UNIX socket.
     * The unix domain socket interface is an alternative to the original magellan
     * protocol, and it is *NOT* compatible with the 3D connexion driver. If you wish
     * to remain compatible, use the X11 protocol (spnav_x11_open, see below).
     */
    pub fn spnav_open() -> Result<(), ()> {
        unsafe {
            if libspnav::spnav_open() == -1 {
                Err(())
            } else {
                Ok(())
            }
        }
    }

    /* Close connection to the daemon. Use it for X11 or AF_UNIX connections.
     * Returns -1 on failure
     */
    // int spnav_close(void);
    pub fn spnav_close() -> Result<(), ()> {
        unsafe {
            if libspnav::spnav_close() == -1 {
                Err(())
            } else {
                Ok(())
            }
        }
    }

    /* Retrieves the file descriptor used for communication with the daemon, for
     * use with select() by the application, if so required.
     * If the X11 mode is used, the socket used to communicate with the X server is
     * returned, so the result of this function is always reliable.
     * If AF_UNIX mode is used, an error is returned if
     * no connection is open / failure occured.
     */
    // int spnav_fd(void);
    pub fn spnav_fd() -> Result<i32, ()> {
        unsafe {
            let fd = libspnav::spnav_fd();
            if fd == -1 {
                Err(())
            } else {
                Ok(fd)
            }
        }
    }

    /* TODO: document */
    // int spnav_sensitivity(double sens);
    pub fn spnav_sensitivity(sens: f64) -> Result<i32, ()> {
        unsafe {
            let v = libspnav::spnav_sensitivity(sens);
            if v == -1 {
                Err(())
            } else {
                Ok(v)
            }
        }
    }

    /* blocks waiting for space-nav events. returns 0 if an error occurs */
    // int spnav_wait_event(spnav_event *event);
    pub fn spnav_wait_event() -> Result<Event, ()> {
        let mut event = libspnav::spnav_event {
            type_: SPNAV_EVENT_ANY,
        };
        let t = unsafe { libspnav::spnav_wait_event(&mut event) };
        if t == 0 {
            Err(())
        } else {
            event.try_into()
        }
    }

    /* checks the availability of space-nav events (non-blocking)
     * returns the event type if available, or 0 otherwise.
     */
    // int spnav_poll_event(spnav_event *event);
    pub fn spnav_poll_event() -> Option<Event> {
        let mut event = libspnav::spnav_event {
            type_: SPNAV_EVENT_ANY,
        };
        let t = unsafe { libspnav::spnav_poll_event(&mut event) };
        if t == 0 {
            None
        } else {
            event.try_into().ok()
        }
    }

    /* Removes any pending events from the specified type, or all pending events
     * events if the type argument is SPNAV_EVENT_ANY. Returns the number of
     * removed events.
     */
    // int spnav_remove_events(int type);
    pub fn spnav_remove_events(t: EventType) -> i32 {
        unsafe { libspnav::spnav_remove_events(t.into()) }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() -> Result<(), ()> {
        let c = Connection::new()?;
        println!("{:?}", c);
        println!("{:?}", c.wait());
        Ok(())
    }
}
