// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// This Source Code Form is "Incompatible With Secondary Licenses", as
// defined by the Mozilla Public License, v. 2.0.
//
// Copyright © 2022 mumblingdrunkard

use super::{
    instruction::{Conclusion, Operation},
    Hart,
};

pub trait Rv32a {
    fn lr_w(&mut self, op: &Operation) -> Conclusion;
    fn sc_w(&mut self, op: &Operation) -> Conclusion;

    fn amoswap_w(&mut self, op: &Operation) -> Conclusion;
    fn amoadd_w(&mut self, op: &Operation) -> Conclusion;
    fn amoxor_w(&mut self, op: &Operation) -> Conclusion;
    fn amoand_w(&mut self, op: &Operation) -> Conclusion;
    fn amoor_w(&mut self, op: &Operation) -> Conclusion;
    fn amomin_w(&mut self, op: &Operation) -> Conclusion;
    fn amomax_w(&mut self, op: &Operation) -> Conclusion;
    fn amominu_w(&mut self, op: &Operation) -> Conclusion;
    fn amomaxu_w(&mut self, op: &Operation) -> Conclusion;
}

impl<'a> Rv32a for Hart<'a> {
    #[inline(always)]
    fn lr_w(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }

    #[inline(always)]
    fn sc_w(&mut self, _op: &Operation) -> Conclusion {
        // NB: "The SC must fail if a store to the reservation set from
        // another hart can be observed to occur between the LR and SC."
        //
        // Does this mean all stores have to affect the reservations?
        //
        // How to work with this without making stores painfully slow
        // in the case that harts aren't acting stupid?
        //
        // Invalidate when cache-lines write back and when atomics store?
        todo!()
    }

    #[inline(always)]
    fn amoswap_w(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }

    #[inline(always)]
    fn amoadd_w(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }

    #[inline(always)]
    fn amoxor_w(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }

    #[inline(always)]
    fn amoand_w(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }

    #[inline(always)]
    fn amoor_w(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }

    #[inline(always)]
    fn amomin_w(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }

    #[inline(always)]
    fn amomax_w(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }

    #[inline(always)]
    fn amominu_w(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }

    #[inline(always)]
    fn amomaxu_w(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }
}
