use crate::desire::{Home, Routed, Work};
use crate::souls::Soul;
use egregoria::api::Router;
use egregoria::map_dynamic::BuildingInfos;
use egregoria::pedestrians::spawn_pedestrian;
use egregoria::utils::rand_provider::RandProvider;
use egregoria::vehicles::spawn_parked_vehicle;
use egregoria::{Egregoria, SoulID};
use map_model::{BuildingID, BuildingKind, Map};

pub type HumanSoul = Soul<Human, (Work, Home)>;

pub struct Human {
    pub(crate) router: Router,
}

impl Routed for Human {
    fn router_mut(&mut self) -> &mut Router {
        &mut self.router
    }
}

impl Human {
    pub fn soul(id: SoulID, house: BuildingID, goria: &mut Egregoria) -> Option<HumanSoul> {
        let map = goria.read::<Map>();
        let work = map
            .random_building(BuildingKind::Workplace, &mut *goria.write::<RandProvider>())?
            .id;
        let housepos = map.buildings()[house].door_pos;
        drop(map);

        goria.write::<BuildingInfos>().add_owner(house, id);

        let body = spawn_pedestrian(goria, house);
        let car = spawn_parked_vehicle(goria, housepos);

        let offset = goria.write::<RandProvider>().random::<f32>() * 0.5;

        let router = Router::new(body, car);

        Some(Soul {
            id,
            desires: (Work::new(work, offset), Home::new(house, offset)),
            extra: Human { router },
        })
    }
}
