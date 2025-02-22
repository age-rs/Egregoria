#![cfg(test)]

use crate::engine_interaction::WorldCommands;
use crate::map_dynamic::BuildingInfos;
use crate::utils::scheduler::SeqSchedule;
use crate::Egregoria;
use common::logger::MyLog;
use geom::Vec2;
use map_model::{BuildingID, LanePatternBuilder};

mod vehicles;

struct TestCtx {
    pub g: Egregoria,
    sched: SeqSchedule,
}

impl TestCtx {
    fn init() -> Self {
        MyLog::init();

        let g = Egregoria::empty();
        let sched = Egregoria::schedule();

        Self { g, sched }
    }

    fn build_roads(&self, v: &[Vec2]) {
        let mut m = self.g.map_mut();
        for w in v.windows(2) {
            let a = m.project(w[0], 0.0);
            let b = m.project(w[1], 0.0);
            m.make_connection(a, b, None, &LanePatternBuilder::default().build());
        }
    }

    fn build_house_near(&self, p: Vec2) -> BuildingID {
        let lot = self
            .g
            .map()
            .lots()
            .values()
            .min_by_key(|lot| lot.shape.center().distance2(p) as i32)
            .unwrap()
            .id;

        let b = self.g.map_mut().build_house(lot).unwrap();
        self.g.write::<BuildingInfos>().insert(b);
        b
    }

    fn tick(&mut self) {
        self.g.tick(&mut self.sched, &WorldCommands::default());
    }
}
