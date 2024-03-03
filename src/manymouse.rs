use std::{
    ffi::{c_char, c_int, c_uint, CStr},
    mem::MaybeUninit,
};

extern "C" {
    fn ManyMouse_Init() -> c_int;
    fn ManyMouse_DriverName() -> *const c_char;
    fn ManyMouse_Quit();
    fn ManyMouse_DeviceName(index: c_uint) -> *const c_char;
    fn ManyMouse_PollEvent(event: *mut Event) -> c_int;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum EventType {
    Absmotion = 0,
    Relmotion,
    Button,
    Scroll,
    Disconnect,
    Max,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub ev_type: EventType,
    pub device: c_uint,
    pub item: c_uint,
    pub value: c_int,
    pub minval: c_int,
    pub maxval: c_int,
}

pub enum Axis {
    X = 0,
    Y,
}

pub enum Button {
    LMB = 0,
    RMB,
}

pub struct EventIter<'a> {
    _mm: &'a mut ManyMouse,
}

impl<'a> Iterator for EventIter<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let mut ev = MaybeUninit::uninit();
            if ManyMouse_PollEvent(ev.as_mut_ptr()) != 0 {
                Some(ev.assume_init())
            } else {
                None
            }
        }
    }
}

pub struct ManyMouse {
    avail_mice_len: u32,
}

impl ManyMouse {
    pub fn new() -> Self {
        unsafe {
            let available_mice = ManyMouse_Init();
            if available_mice == -1 {
                ManyMouse_Quit();
                panic!("ManyMouse couldn't initialize");
            }
            Self {
                avail_mice_len: available_mice as u32,
            }
        }
    }

    pub fn poll(&mut self) -> EventIter {
        EventIter { _mm: self }
    }

    pub fn device_list(&self) -> Vec<String> {
        unsafe {
            (0..self.avail_mice_len)
                .map(|i| {
                    CStr::from_ptr(ManyMouse_DeviceName(i))
                        .to_string_lossy()
                        .to_string()
                })
                .collect()
        }
    }

    pub fn driver_name(&self) -> String {
        unsafe {
            CStr::from_ptr(ManyMouse_DriverName())
                .to_string_lossy()
                .to_string()
        }
    }
}

impl Drop for ManyMouse {
    fn drop(&mut self) {
        unsafe {
            ManyMouse_Quit();
        }
    }
}
