use std::ptr::{read_volatile, write_volatile};
use std::thread;

const NUM_THREADS: usize = 8;
const NUM_LOOP: usize = 1_000_000;

static mut SUM: u64 = 0;
static mut ENTERING: [bool; NUM_THREADS] = [false; NUM_THREADS];
static mut TICKETS: [Option<u64>; NUM_THREADS] = [None; NUM_THREADS];

unsafe fn bakery_lock_acq(idx: usize) {
    write_volatile(&mut ENTERING[idx], true);
    
    let ticket = 1 + TICKETS.iter()
        .filter_map(|&ticket| ticket)
        .max().unwrap_or(0);
    write_volatile(&mut TICKETS[idx], Some(ticket));

    write_volatile(&mut ENTERING[idx], false);

    for i in 0..NUM_THREADS {
        if i == idx { 
            continue;
        }

        while read_volatile(&ENTERING[i]) { }

        loop {
            if let Some(t) = read_volatile(&TICKETS[i]) {
                if ticket < t || (ticket == t && idx < i) {
                    break;
                }
            }
            else {
                break;
            }
        }
    }
}

unsafe fn bakery_lock_rel(idx: usize) {
    write_volatile(&mut TICKETS[idx], None);
}


fn main() {
    let mut v = Vec::with_capacity(NUM_THREADS);
    for id in 0..NUM_THREADS {
        v.push(thread::spawn(move || unsafe {
            for _ in 0..NUM_LOOP {
                bakery_lock_acq(id);
                let num = read_volatile(&SUM);
                write_volatile(&mut SUM, num + 1);
                bakery_lock_rel(id);
            }
        }));
    }

    for th in v {
        th.join().unwrap();
    }

    println!("SUM={}, (expected={})", unsafe { SUM }, NUM_THREADS * NUM_LOOP);
}