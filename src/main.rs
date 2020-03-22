extern crate crossbeam_channel as channel;
extern crate ignore;
extern crate walkdir;

use ignore::{WalkBuilder, WalkState};
use std::path::PathBuf;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use std::thread;

mod channel_vs_hashmap;

fn main() {
    let args: Vec<String> = env::args().collect();

    let dir = {
        if args.len() < 2 {
            "."
        } else {
            &args[1]
        }
    };
    println!("Analizing {}", dir);

    // Using a channel was ~ 10% faster than using a shared hashmap
    //channel_vs_hashmap::walk_para_hash_map(dir);
    //channel_vs_hashmap::walk_para_channel(dir);

    // _in_rec 'doing the work in the single receiver thread' was aprox twice as fast
    walk_para_channel_in_rec(dir);
    //walk_para_channel_in_tx(dir);

    // Introducing 2 reader threads killed performance, probably all the lock spinning
    //walk_para_channel_in_rec_para(dir);

    // Tried 2 reader threads each building their own hashmap but it looks like most of the work 
    // is actually being done in the write threads.
}

fn walk_para_channel_in_tx(dir: &str) {
    let (tx, rx) = channel::bounded::<(PathBuf, u64)>(1000);

    // Receiver thread
    let stdout_thread = thread::spawn(move || {
        let mut hash: HashMap<String, u64> = HashMap::new();
        for dent in rx {
            let (path, size) = dent;
            let s = hash.entry(path.to_str().unwrap().to_string()).or_insert(0);
            *s += size;
        }
        hash
    });

    // Walk the dir tree thread
    let builder = WalkBuilder::new(dir);
    builder.build_parallel().run(|| {
        let txc = tx.clone();
        Box::new(move |path| {
            match path {
                Ok(p) => {
                    let u = p.metadata().unwrap().len();
                    for pa in p.into_path().ancestors() {
                        txc.send((pa.to_path_buf(), u) ).unwrap();
                    }
                },
                Err(_) => {}
            };
            WalkState::Continue
        })
    });

    drop(tx);
    let hash = stdout_thread.join().unwrap();
    println!("Found {:?} files ", hash.len());
    //println!("{:?}", hash);
}

fn walk_para_channel_in_rec(dir: &str) {
    let (tx, rx) = channel::bounded::<(PathBuf, u64)>(1000);

    // Receiver thread
    let stdout_thread = thread::spawn(move || {
        let mut hash: HashMap<String, u64> = HashMap::new();
        for dent in rx {
            let (path, size) = dent;
            for p in path.ancestors() {
                let s = hash.entry(p.to_str().unwrap().to_string()).or_insert(0);
                *s += size;
            }
        }
        hash
    });

    // Walk the dir tree thread
    let builder = WalkBuilder::new(dir);
    builder.build_parallel().run(|| {
        let txc = tx.clone();
        Box::new(move |path| {
            match path {
                Ok(p) => {
                    let u = p.metadata().unwrap().len();
                    txc.send((p.into_path(), u) ).unwrap();
                },
                Err(_) => {}
            };
            WalkState::Continue
        })
    });

    drop(tx);
    let hash = stdout_thread.join().unwrap();
    println!("Found {:?} files ", hash.len());
    //println!("{:?}", hash);
}

fn walk_para_channel_in_rec_para(dir: &str) {
    let (tx, rx) = channel::bounded::<(PathBuf, u64)>(1000);
    let rx2 = rx.clone();

    let hash: HashMap<String, u64> = HashMap::new();
    let mutex_hash = Arc::new(Mutex::new(hash));
    // let mutex_hash2 = mutex_hash.clone();
    let mh = mutex_hash.clone();

    let hash: HashMap<String, u64> = HashMap::new();
    let mutex_hash2 = Arc::new(Mutex::new(hash));

    // Receiver thread
    let stdout_thread = thread::spawn(move || {
        let local_hash = mutex_hash.clone();
        for dent in rx {
            let (path, size) = dent;
            let mut h = local_hash.lock().unwrap();
            for p in path.ancestors() {
                let s = h.entry(p.to_str().unwrap().to_string()).or_insert(0);
                *s += size;
            }
        }
    });

    // gratuotus copy paste coding
    let stdout_thread2 = thread::spawn(move || {
        let local_hash = mutex_hash2.clone();
        for dent in rx2 {
            let (path, size) = dent;
            let mut h = local_hash.lock().unwrap();
            for p in path.ancestors() {
                let s = h.entry(p.to_str().unwrap().to_string()).or_insert(0);
                *s += size;
            }
        }
    });

    // Walk the dir tree thread
    let builder = WalkBuilder::new(dir);
    builder.build_parallel().run(|| {
        let txc = tx.clone();
        Box::new(move |path| {
            match path {
                Ok(p) => {
                    let u = p.metadata().unwrap().len();
                    txc.send((p.into_path(), u)).unwrap();
                },
                Err(_) => {}
            };
            WalkState::Continue
        })
    });

    drop(tx);

    stdout_thread.join().unwrap();
    stdout_thread2.join().unwrap();
    println!("Found {:?} files ", mh.lock().unwrap().len());
    //println!("{:?}", hash);
}
