#![allow(unused_parens)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(non_upper_case_globals)]

use crate::faust::*;

include!(concat!(env!("OUT_DIR"), "/sine.rs"));
include!(concat!(env!("OUT_DIR"), "/noise.rs"));
