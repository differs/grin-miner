// Copyright 2017 The Grin Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Tests exercising the loading and unloading of plugins, as well as the
/// existence and correct functionality of each plugin function

extern crate rand;
extern crate cuckoo_miner as cuckoo;

use std::path::PathBuf;
use std::{thread, time};
use std::time::Instant;

use cuckoo::{SolverCtx, SolverParams, SolverStats, SolverSolutions, CuckooMinerError, PluginLibrary};

/*pub mod common;

use common::{
	KNOWN_30_HASH_1,
	KNOWN_16_HASH_1};*/

static DLL_SUFFIX: &str = ".cuckooplugin";


/// Solution for 80 length header with nonce 0
const CUCKATOO_29_SOL:[u64; 42] = [
0x48a9e2, 0x9cf043, 0x155ca30, 0x18f4783, 0x248f86c, 0x2629a64, 0x5bad752, 0x72e3569,
0x93db760, 0x97d3b37, 0x9e05670, 0xa315d5a, 0xa3571a1, 0xa48db46, 0xa7796b6, 0xac43611,
0xb64912f, 0xbb6c71e, 0xbcc8be1, 0xc38a43a, 0xd4faa99, 0xe018a66, 0xe37e49c, 0xfa975fa,
0x11786035, 0x1243b60a, 0x12892da0, 0x141b5453, 0x1483c3a0, 0x1505525e, 0x1607352c, 0x16181fe3, 
0x17e3a1da, 0x180b651e, 0x1899d678, 0x1931b0bb, 0x19606448, 0x1b041655, 0x1b2c20ad, 0x1bd7a83c, 0x1c05d5b0, 0x1c0b9caa ];

const TEST_PLUGIN_LIBS_CORE : [&str;2] = [
	"cuckatoo_mean_compat_cpu_19",
	"cuckatoo_mean_compat_cpu_29",
];

const TEST_PLUGIN_LIBS_OPTIONAL : [&str;1] = [
	"cuckatoo_cuda_30",
];

//Helper to convert from hex string
fn from_hex_string(in_str: &str) -> Vec<u8> {
	let mut bytes = Vec::new();
	for i in 0..(in_str.len() / 2) {
		let res = u8::from_str_radix(&in_str[2 * i..2 * i + 2], 16);
		match res {
			Ok(v) => bytes.push(v),
			Err(e) => println!("Problem with hex: {}", e),
		}
	}
	bytes
}

//Helper to load a plugin library
fn load_plugin_lib(plugin:&str) -> Result<PluginLibrary, CuckooMinerError> {
	let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	d.push(format!("../target/debug/plugins/{}{}", plugin, DLL_SUFFIX).as_str());
	PluginLibrary::new(d.to_str().unwrap())
}

//Helper to load all plugin libraries specified above
fn load_all_plugins() -> Vec<PluginLibrary>{
	let mut plugin_libs:Vec<PluginLibrary> = Vec::new();
	for p in TEST_PLUGIN_LIBS_CORE.into_iter(){
		plugin_libs.push(load_plugin_lib(p).unwrap());
	}
	for p in TEST_PLUGIN_LIBS_OPTIONAL.into_iter(){
		let pl = load_plugin_lib(p);
		if let Ok(p) = pl {
			plugin_libs.push(p);
		}
	}
	plugin_libs
}

//loads and unloads a plugin many times
#[test]
fn on_commit_plugin_loading(){
	//core plugins should be built on all systems, fail if they don't exist
	for _ in 0..100 {
		for p in TEST_PLUGIN_LIBS_CORE.into_iter() {
			let pl = load_plugin_lib(p).unwrap();
			pl.unload();
		}
	}
	//only test these if they do exist (cuda, etc)
	for _ in 0..100 {
		for p in TEST_PLUGIN_LIBS_OPTIONAL.into_iter() {
			let pl = load_plugin_lib(p);
			if let Err(_) = pl {
				break;
			}
			pl.unwrap().unload();
		}
	}
}

//Loads all plugins at once
#[test]
fn plugin_multiple_loading(){
	let _p=load_all_plugins();
}

#[test]
fn sanity_cuckatoo_mean_compat_cpu_29() {
	let pl = load_plugin_lib("cuckatoo_mean_compat_cpu_29").unwrap();
	let mut params = SolverParams::default();
	let mut sols = SolverSolutions::default();
	let mut stats = SolverStats::default();
	params.nthreads = 4;
	// to be consistent with command line solver operation
	params.mutate_nonce = true;
	let ctx = pl.create_solver_ctx(&mut params);
	let test_header = [0u8; 80].to_vec();
	let res = pl.run_solver(
		ctx,
		test_header,
		20,
		1,
		&mut sols,
		&mut stats,
		);
	assert_eq!(sols.num_sols, 1);
	assert_eq!(sols.edge_bits, 29);
	assert_eq!(stats.edge_bits, 29);
	for i in 0..42 {
		assert_eq!(sols.sols[0].proof[i], CUCKATOO_29_SOL[i]);
	}
	pl.destroy_solver_ctx(ctx);
	pl.unload();
}

#[cfg(feature="build-cuda-plugins")]
#[test]
fn sanity_cuckatoo_cuda_29() {
	let pl = load_plugin_lib("cuckatoo_cuda_29").unwrap();
	let mut params = pl.get_default_params();
	let mut sols = SolverSolutions::default();
	let mut stats = SolverStats::default();
	params.expand = 1;
	// to be consistent with command line solver operation
	params.mutate_nonce = true;
	let ctx = pl.create_solver_ctx(&mut params);
	let test_header = [0u8; 80].to_vec();
	let res = pl.run_solver(
		ctx,
		test_header,
		20,
		1,
		&mut sols,
		&mut stats,
		);
	assert_eq!(sols.num_sols, 1);
	assert_eq!(sols.edge_bits, 29);
	assert_eq!(stats.edge_bits, 29);
	for i in 0..42 {
		assert_eq!(sols.sols[0].proof[i], CUCKATOO_29_SOL[i]);
	}
	pl.destroy_solver_ctx(ctx);
	pl.unload();
}
