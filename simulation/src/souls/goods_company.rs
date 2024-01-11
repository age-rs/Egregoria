use serde::{Deserialize, Serialize};

use egui_inspect::Inspect;
use geom::{Transform, Vec2};
use prototypes::{CompanyKind, GoodsCompanyID, GoodsCompanyPrototype, ItemID, Recipe};

use crate::economy::{find_trade_place, Market};
use crate::map::{Building, BuildingID, Map, Zone, MAX_ZONE_AREA};
use crate::map_dynamic::BuildingInfos;
use crate::souls::desire::WorkKind;
use crate::utils::resources::Resources;
use crate::utils::time::GameTime;
use crate::world::{CompanyEnt, HumanEnt, HumanID, VehicleID};
use crate::{ParCommandBuffer, SoulID};
use crate::{Simulation, World};

use super::desire::Work;

pub fn recipe_init(recipe: &Recipe, soul: SoulID, near: Vec2, market: &mut Market) {
    for item in &recipe.consumption {
        market.buy_until(soul, near, item.id, item.amount as u32)
    }
    for item in &recipe.production {
        market.register(soul, item.id);
    }
}

pub fn recipe_should_produce(recipe: &Recipe, soul: SoulID, market: &Market) -> bool {
    // Has enough resources
    recipe.consumption
            .iter()
            .all(move |item| market.capital(soul, item.id) >= item.amount)
            &&
            // Has enough storage
            recipe.production.iter().all(move |item| {
                market.capital(soul, item.id) < item.amount * (recipe.storage_multiplier + 1)
            })
}

pub fn recipe_act(recipe: &Recipe, soul: SoulID, near: Vec2, market: &mut Market) {
    for item in &recipe.consumption {
        market.produce(soul, item.id, -item.amount);
        market.buy_until(soul, near, item.id, item.amount as u32);
    }
    for item in &recipe.production {
        market.produce(soul, item.id, item.amount);
        market.sell_all(
            soul,
            near,
            item.id,
            (item.amount * recipe.storage_multiplier) as u32,
        );
    }
}

#[derive(Clone, Serialize, Deserialize, Inspect)]
pub struct GoodsCompanyState {
    pub kind: GoodsCompanyID,
    pub building: BuildingID,
    pub max_workers: i32,
    /// In [0; 1] range, to show how much has been made until new product
    pub progress: f32,
    pub driver: Option<HumanID>,
    pub trucks: Vec<VehicleID>,
}

impl GoodsCompanyState {
    pub fn productivity(&self, workers: usize, zone: Option<&Zone>) -> f32 {
        workers as f32 / self.max_workers as f32 * zone.map_or(1.0, |z| z.area / MAX_ZONE_AREA)
    }
}

pub fn company_soul(
    sim: &mut Simulation,
    company: GoodsCompanyState,
    proto: &GoodsCompanyPrototype,
) -> Option<SoulID> {
    let map = sim.map();
    let b = map.buildings().get(company.building)?;
    let door_pos = b.door_pos;
    let obb = b.obb;
    let height = b.height;
    drop(map);

    let id = sim.world.insert(CompanyEnt {
        trans: Transform::new(obb.center().z(height)),
        comp: company,
        workers: Default::default(),
        sold: Default::default(),
        bought: Default::default(),
    });

    let company = &sim.world.get(id).unwrap().comp;

    let soul = SoulID::GoodsCompany(id);

    let job_opening = ItemID::new("job-opening");

    {
        let m = &mut *sim.write::<Market>();
        m.produce(soul, job_opening, company.max_workers);
        m.sell_all(soul, door_pos.xy(), job_opening, 0);

        recipe_init(&proto.recipe, soul, door_pos.xy(), m);
    }

    sim.write::<BuildingInfos>()
        .set_owner(company.building, soul);

    Some(soul)
}

pub fn company_system(world: &mut World, res: &mut Resources) {
    profiling::scope!("souls::company_system");
    let delta = res.read::<GameTime>().realdelta;
    let cbuf: &ParCommandBuffer<CompanyEnt> = &res.read();
    let cbuf_human: &ParCommandBuffer<HumanEnt> = &res.read();
    let binfos: &BuildingInfos = &res.read();
    let market: &Market = &res.read();
    let map: &Map = &res.read();

    world.companies.iter_mut().for_each(|(me, c)| {
        let n_workers = c.workers.0.len();
        let soul = SoulID::GoodsCompany(me);
        let b: &Building = unwrap_or!(map.buildings.get(c.comp.building), {
            cbuf.kill(me);
            return;
        });

        let proto = c.comp.kind.prototype();

        if recipe_should_produce(&proto.recipe, soul, market) {
            c.comp.progress += c.comp.productivity(n_workers, b.zone.as_ref())
                / proto.recipe.complexity as f32
                * delta;
        }

        if c.comp.progress >= 1.0 {
            c.comp.progress -= 1.0;
            let kind = c.comp.kind;
            let bpos = b.door_pos;

            cbuf.exec_on(me, move |market| {
                let recipe = &kind.prototype().recipe;
                recipe_act(recipe, soul, bpos.xy(), market);
            });
            return;
        }

        for (_, trades) in c.bought.0.iter_mut() {
            for trade in trades.drain(..) {
                if let Some(owner_build) =
                    find_trade_place(trade.seller, b.door_pos.xy(), binfos, map)
                {
                    cbuf.exec_ent(me, move |sim| {
                        let (world, res) = sim.world_res();
                        if let Some(SoulID::FreightStation(owner)) =
                            res.read::<BuildingInfos>().owner(owner_build)
                        {
                            if let Some(f) = world.freight_stations.get_mut(owner) {
                                f.f.wanted_cargo += 1;
                            }
                        }
                    });
                }
            }
        }

        (|| {
            let Some(trade) = c.sold.0.pop() else {
                return;
            };
            let Some(driver) = c.comp.driver else {
                return;
            };
            let Some(w) = world.humans.get(driver).and_then(|h| h.work.as_ref()) else {
                return;
            };
            if !matches!(
                w.kind,
                WorkKind::Driver {
                    deliver_order: None,
                    ..
                }
            ) {
                return;
            }
            let Some(owner_build) = find_trade_place(trade.buyer, b.door_pos.xy(), binfos, map)
            else {
                log::warn!("driver can't find the place to deliver for {:?}", &trade);
                return;
            };
            cbuf.exec_ent(me, move |sim| {
                let Some(h) = sim.world.humans.get_mut(driver) else {
                    return;
                };
                let Some(w) = h.work.as_mut() else {
                    return;
                };
                let WorkKind::Driver { deliver_order, .. } = &mut w.kind else {
                    return;
                };
                *deliver_order = Some(owner_build)
            });
        })();

        for &worker in c.workers.0.iter() {
            let Some(w) = world.humans.get(worker) else {
                continue;
            };

            if w.work.is_none() {
                let mut kind = WorkKind::Worker;

                if let Some(truck) = c.comp.trucks.get(0) {
                    if proto.kind == CompanyKind::Factory && c.comp.driver.is_none() {
                        kind = WorkKind::Driver {
                            deliver_order: None,
                            truck: *truck,
                        };

                        c.comp.driver = Some(worker);
                    }
                }

                let offset = common::rand::randu(common::hash_u64(worker) as u32);

                let b = c.comp.building;
                cbuf_human.exec_ent(worker, move |sim| {
                    let Some(w) = sim.world.humans.get_mut(worker) else {
                        return;
                    };
                    w.work = Some(Work::new(b, kind, offset));
                });
            }
        }
    });
}
