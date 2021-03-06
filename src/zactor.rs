//! Module: czmq-zactor

use {czmq_sys, Error, ErrorKind, RawInterface, Result, Sockish, ZMsg, ZSock};
use std::{error, fmt, ptr};
use std::os::raw::c_void;

pub struct ZActor {
    zactor: *mut czmq_sys::zactor_t,
    owned: bool,
}

unsafe impl Send for ZActor {}

impl Drop for ZActor {
    fn drop(&mut self) {
        if self.owned {
            unsafe { czmq_sys::zactor_destroy(&mut self.zactor) };
        }
    }
}

impl ZActor {
    pub fn new(task: czmq_sys::zactor_fn) -> Result<ZActor> {
        let zactor = unsafe { czmq_sys::zactor_new(task, ptr::null_mut()) };

        if zactor == ptr::null_mut() {
            Err(Error::new(ErrorKind::NullPtr, ZActorError::Instantiate))
        } else {
            Ok(ZActor {
                zactor: zactor,
                owned: true,
            })
        }
    }

    pub fn send(&self, msg: ZMsg) -> Result<()> {
        let rc = unsafe { czmq_sys::zactor_send(self.zactor, &mut msg.into_raw()) };
        if rc == -1 {
            Err(Error::new(ErrorKind::NonZero, ZActorError::CmdFailed))
        } else {
            Ok(())
        }
    }

    pub fn send_str(&self, string: &str) -> Result<()> {
        let msg = ZMsg::new();
        try!(msg.addstr(string));
        self.send(msg)
    }

    pub fn recv(&self) -> Result<ZMsg> {
        let zmsg_ptr = unsafe { czmq_sys::zactor_recv(self.zactor) };

        if zmsg_ptr == ptr::null_mut() {
            Err(Error::new(ErrorKind::NullPtr, ZActorError::CmdFailed))
        } else {
            unsafe { Ok(ZMsg::from_raw(zmsg_ptr, true)) }
        }
    }

    pub fn sock(&self) -> ZSock {
        unsafe { ZSock::from_raw(czmq_sys::zactor_sock(self.zactor) as *mut c_void, false) }
    }
}

impl RawInterface<c_void> for ZActor {
    unsafe fn from_raw(ptr: *mut c_void, owned: bool) -> ZActor {
        ZActor {
            zactor: ptr as *mut czmq_sys::zactor_t,
            owned: owned,
        }
    }

    fn into_raw(self) -> *mut c_void {
        self.zactor as *mut c_void
    }

    fn as_mut_ptr(&mut self) -> *mut c_void {
        self.zactor as *mut c_void
    }
}

impl Sockish for ZActor {}

#[derive(Debug)]
pub enum ZActorError {
    Instantiate,
    CmdFailed,
}

impl fmt::Display for ZActorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ZActorError::Instantiate => write!(f, "Could not instantiate new ZActor struct"),
            ZActorError::CmdFailed => write!(f, "ZActor command failed"),
        }
    }
}

impl error::Error for ZActorError {
    fn description(&self) -> &str {
        match *self {
            ZActorError::Instantiate => "Could not instantiate new ZActor struct",
            ZActorError::CmdFailed => "ZActor command failed",
        }
    }
}
