// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use {
        pemios_core::bus::Bus,
        pemios_core::hart::{
            instruction::{Conclusion, Execute},
            Reg,
        },
    };

    #[test]
    fn fib() {
        // decoded instructions
        use pemios_core::hart::Hart;
        use std::fs;

        let program = fs::read("resources/test_programs/fib").unwrap();

        let bus = Arc::new(Bus::new(1024));
        if let Err(_) = bus.set_mm(&program) {
            todo!();
        };

        let mut h = Hart::new(bus);

        h.reg.set(Reg::SP, 0x1000 << 10);

        let start = std::time::Instant::now();

        let mut ctr = 0;
        loop {
            ctr += 1;
            if let Conclusion::Exception(_) = h.execute() {
                println!("Done with {ctr} instructions! Result: {}", h.reg[Reg::A1]);
                break;
            }
        }

        let end = std::time::Instant::now();

        println!("pre-decoding took: {:?}", end - start);
    }
}
