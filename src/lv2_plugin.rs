#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

extern crate libc;
extern crate lv2;
use std::ffi::CStr;
use std::ffi::CString;
use std::ptr;
use plugin;
use midi;
use midi::*;
use plugin::*;
use std::str;

pub struct Synthuris {
    pub midi_event: lv2::LV2Urid
}

impl Synthuris {
    fn new() -> Synthuris {
        Synthuris {
            midi_event: 0 as lv2::LV2Urid
        }
    }
}

#[repr(C)]
pub struct Lv2SynthPlugin {
    pub map: *mut lv2::LV2UridMap,
    pub in_port: *const lv2::LV2_Atom_Sequence,
    pub output: *mut f32,
    pub uris: Synthuris,
    pub plugin: plugin::SynthPlugin,
}

impl  Lv2SynthPlugin {
    pub fn new() -> Lv2SynthPlugin {
        // let np = ptr::null();
        let mut lv2plugin = Lv2SynthPlugin {
            map: ptr::null_mut(),
            in_port: ptr::null(),
            output: ptr::null_mut(),
            uris: Synthuris::new(),
            plugin: plugin::SynthPlugin::new(),
        };
        // TODO: this is to avoid needing to access lv2plugin.plugin in lv2::LV2Descriptor::connect_port()
        lv2plugin.output = lv2plugin.plugin.audio_out;
        lv2plugin
    }
    pub fn seturis(&mut self) {
        unsafe{
            let s = "http://lv2plug.in/ns/ext/midi#MidiEvent";
            let cstr = CString::new(s).unwrap();
            let lv2_midi_midi_event = cstr.as_ptr();
            self.uris.midi_event = ((*self.map).map)((*self.map).handle, lv2_midi_midi_event);
        }
    }
    pub fn connect_port(&mut self, port: u32, data: *mut libc::c_void) {
        match port {
            0 => self.in_port = data  as *const lv2::LV2_Atom_Sequence,
            1 => self.output = data as *mut f32,
            _ => self.map_params(port,data)
        }
    }
    // fn get_nparams(&self) -> u32 {
    //     self.plugin.params.len()
    // }
    pub fn midievent(&mut self, msg: &u8) {
        let mm = msg as midi::MidiMessage;
        self.plugin.midievent(mm)
    }
    pub fn set_fs(&mut self, fs: f64) {
        self.plugin.set_fs(fs);
    }
    pub fn get_amp(&mut self) -> f32 {
        self.plugin.get_amp()
    }
    fn map_params(&mut self, port: u32, data: *mut libc::c_void) {
        let nparams = 1;
        let iport = port - 2; //TODO: don't hardcode number of input/output ports
        if iport <= nparams-1 {
            println!("connecting port: {}", port);
            self.plugin.params[iport as usize]= data  as *mut f32 ;
            // println!("param: {}",  *(self.synth.params[0]));
        } else {
            panic!("Not a valid PortIndex: {}", iport)
        }
    }
}
