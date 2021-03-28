use crate::{Lane, LaneID, LaneKind, CROSSWALK_WIDTH};
use geom::{Transform, Vec2};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

new_key_type! {
    pub struct ParkingSpotID;
}

pub const PARKING_SPOT_LENGTH: f32 = 6.0;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct ParkingSpot {
    pub parent: LaneID,
    pub trans: Transform,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ParkingSpots {
    spots: SlotMap<ParkingSpotID, ParkingSpot>,
    lane_spots: SecondaryMap<LaneID, Vec<ParkingSpotID>>,
}

impl ParkingSpots {
    pub fn get(&self, spot: ParkingSpotID) -> Option<&ParkingSpot> {
        self.spots.get(spot)
    }

    pub fn contains(&self, spot: ParkingSpotID) -> bool {
        self.spots.contains_key(spot)
    }

    pub fn remove_spots(&mut self, lane: LaneID) {
        if let Some(spots) = self.lane_spots.remove(lane) {
            for spot in spots {
                self.spots.remove(spot);
            }
        }
    }

    pub fn generate_spots(&mut self, lane: &Lane) {
        debug_assert!(matches!(lane.kind, LaneKind::Parking));

        match self.lane_spots.get_mut(lane.id) {
            Some(_) => {}
            None => {
                let gap = CROSSWALK_WIDTH + 8.0;
                let l = lane.length() - gap * 2.0;
                let n_spots = (l / PARKING_SPOT_LENGTH) as i32;
                let step = l / n_spots as f32;

                let parent = lane.id;
                let spots = &mut self.spots;
                let spots = lane
                    .points
                    .points_dirs_along((0..n_spots).map(|x| (x as f32 + 0.5) * step + gap))
                    .map(move |(pos, dir)| {
                        spots.insert(ParkingSpot {
                            parent,
                            trans: Transform::new_cos_sin(pos, dir),
                        })
                    })
                    .collect();

                self.lane_spots.insert(lane.id, spots);
            }
        };
    }

    pub fn clear(&mut self) {
        self.spots.clear();
        self.lane_spots.clear();
    }

    pub fn spots(&self, lane: LaneID) -> impl Iterator<Item = &ParkingSpot> + '_ {
        self.lane_spots
            .get(lane)
            .map(move |x| x.iter().flat_map(move |spot| self.spots.get(*spot)))
            .into_iter()
            .flatten()
    }

    pub fn all_spots(&self) -> impl Iterator<Item = (ParkingSpotID, &ParkingSpot)> + '_ {
        self.spots.iter()
    }

    // Fixme: Instead of allocating a vec and sorting it, somehow sort the parking spots beforehand and iterate in spiral around the projected `near`
    pub fn closest_spots(&self, lane: LaneID, near: Vec2) -> impl Iterator<Item = ParkingSpotID> {
        let spots = &self.spots;
        let mut lspots = self.lane_spots.get(lane).cloned();
        if let Some(ref mut lspots) = lspots {
            lspots.sort_by_key(|&id| OrderedFloat(spots[id].trans.position().distance2(near)))
        }
        lspots.into_iter().flatten()
    }
}
