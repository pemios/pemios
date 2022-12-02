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

pub trait Zicsr {
    fn csrrw(&mut self, op: &Operation) -> Conclusion;
    fn csrrs(&mut self, op: &Operation) -> Conclusion;
    fn csrrc(&mut self, op: &Operation) -> Conclusion;
    fn csrrwi(&mut self, op: &Operation) -> Conclusion;
    fn csrrsi(&mut self, op: &Operation) -> Conclusion;
    fn csrrci(&mut self, op: &Operation) -> Conclusion;
}

impl Zicsr for Hart {
    #[inline(always)]
    fn csrrw(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }

    #[inline(always)]
    fn csrrs(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }

    #[inline(always)]
    fn csrrc(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }

    #[inline(always)]
    fn csrrwi(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }

    #[inline(always)]
    fn csrrsi(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }

    #[inline(always)]
    fn csrrci(&mut self, _op: &Operation) -> Conclusion {
        todo!()
    }
}
