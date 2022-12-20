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
    use std::{cell::Cell, sync::atomic::AtomicU32, thread};

    use pemios_core::{
        hart::{instruction::Instruction, step::Step},
        memory::{self, mapping::Mapping},
    };

    use {
        pemios_core::bus::Bus,
        pemios_core::hart::{instruction::Conclusion, Reg},
    };

    #[test]
    fn mt_fib() {
        struct Device<'a> {
            bus: Cell<Option<&'a Bus<'a>>>,
            mem: memory::main::Main<'a>,
        }

        impl<'a> Device<'a> {
            fn new() -> Self {
                Self {
                    bus: Cell::new(None),
                    mem: memory::main::Main::new(0, 1),
                }
            }

            fn set_bus(&'a self, bus: &'a Bus<'a>) {
                self.bus.set(Some(bus));
            }
        }

        use pemios_core::hart::Hart;
        use std::fs;

        let program = fs::read("resources/test_programs/fib").unwrap();

        let (_, tmp, _) = unsafe { program.align_to::<u32>() };
        let test = tmp
            .iter()
            .map(|v| Instruction::from(*v))
            .collect::<Vec<_>>();

        for (a, i) in test.iter().enumerate() {
            let pc = a * 4;
            println!("{pc}: {i:?}");
        }

        let device = Device::new();

        let bus = &Bus::builder()
            .with_main_memory(2)
            .with_mapping(&device.mem)
            .build();

        device.set_bus(bus);

        if bus.set_mm(&program).is_err() {
            todo!();
        };

        let reservation1 = &AtomicU32::new(0xffffffff);

        thread::scope(|s| {
            s.spawn(|| {
                let mut h = Hart::new(bus, reservation1);
                bus.register_reservation_set(reservation1);
                // bus.register_reservation_invalidation(0, &h.reservation);
                h.reg[Reg::SP] = 0x1000;

                let start = std::time::Instant::now();

                loop {
                    if let Conclusion::Exception(_) = h.step() {
                        break;
                    }
                }

                let end = std::time::Instant::now();

                println!("{:?}", h.reg[Reg::A1]);
                println!("fib: took {:?}", end - start);
            });
        });
    }
}
