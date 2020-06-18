use crate::engine_interaction::TimeInfo;
use crate::geometry::intersections::{both_dist_to_inter, Ray};
use crate::geometry::{angle_lerp, Vec2};
use crate::map_model::{
    DirectionalPath, Itinerary, LaneKind, Map, TrafficBehavior, Traversable, TraverseDirection,
    TraverseKind, OBJECTIVE_OK_DIST,
};
use crate::physics::{Collider, CollisionWorld, PhysicsGroup, PhysicsObject};
use crate::physics::{Kinematics, Transform};
use crate::utils::Restrict;
use crate::vehicles::VehicleComponent;
use rand::thread_rng;
use specs::prelude::*;
use specs::shred::PanicHandler;

#[derive(Default)]
pub struct VehicleDecision;

#[derive(SystemData)]
pub struct VehicleDecisionSystemData<'a> {
    map: Read<'a, Map>,
    time: Read<'a, TimeInfo>,
    coworld: Read<'a, CollisionWorld, PanicHandler>,
    colliders: ReadStorage<'a, Collider>,
    transforms: WriteStorage<'a, Transform>,
    kinematics: WriteStorage<'a, Kinematics>,
    vehicles: WriteStorage<'a, VehicleComponent>,
}

impl<'a> System<'a> for VehicleDecision {
    type SystemData = VehicleDecisionSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let cow = data.coworld;
        let map = &*data.map;
        let time = data.time;

        (
            &mut data.transforms,
            &mut data.kinematics,
            &mut data.vehicles,
            &data.colliders,
        )
            .join()
            .for_each(|(trans, kin, vehicle, collider)| {
                objective_update(&mut vehicle.itinerary, &time, trans, &map);

                let (_, self_obj) = cow.get(collider.0).unwrap();
                let speed = self_obj.speed;
                let danger_length = (speed * speed / (2.0 * vehicle.kind.deceleration())).min(40.0);
                let neighbors = cow.query_around(trans.position(), 12.0 + danger_length);
                let objs = neighbors.map(|(id, pos)| (Vec2::from(pos), cow.get(id).unwrap().1));

                calc_decision(vehicle, map, &time, trans, self_obj, objs);

                physics(&time, trans, kin, vehicle, speed);
            });
    }
}

fn physics(
    time: &TimeInfo,
    trans: &mut Transform,
    kin: &mut Kinematics,
    vehicle: &mut VehicleComponent,
    speed: f32,
) {
    let kind = vehicle.kind;
    let direction = trans.direction();

    let speed = speed
        + (vehicle.desired_speed - speed).restrict(
            -time.delta * kind.deceleration(),
            time.delta * kind.acceleration(),
        );

    let max_ang_vel = (speed.abs() / kind.min_turning_radius()).restrict(0.0, 2.0);

    let approx_angle = direction.distance(vehicle.desired_dir);

    vehicle.ang_velocity += time.delta * kind.ang_acc();
    vehicle.ang_velocity = vehicle
        .ang_velocity
        .min(3.0 * approx_angle)
        .min(max_ang_vel);

    trans.set_direction(angle_lerp(
        trans.direction(),
        vehicle.desired_dir,
        vehicle.ang_velocity * time.delta,
    ));

    kin.velocity = trans.direction() * speed;
}

pub fn objective_update(itinerary: &mut Itinerary, time: &TimeInfo, trans: &Transform, map: &Map) {
    itinerary.update(trans.position(), time, map);

    if itinerary.has_ended(time.time) {
        let mut last_travers = itinerary.get_travers().copied();
        if last_travers.is_none() {
            last_travers = map
                .closest_lane(trans.position(), LaneKind::Driving)
                .map(|x| Traversable::new(TraverseKind::Lane(x), TraverseDirection::Forward));
        }

        *itinerary = next_objective(trans.position(), map, last_travers.as_ref())
            .unwrap_or_else(|| Itinerary::wait_until(time.time + 10.0));
    }
}

fn next_objective(pos: Vec2, map: &Map, last_travers: Option<&Traversable>) -> Option<Itinerary> {
    let l = map.get_random_lane(LaneKind::Driving, &mut thread_rng())?;

    Itinerary::route(
        pos,
        *last_travers?,
        (l.id, l.points.random_along().unwrap()),
        map,
        &DirectionalPath,
    )
}

pub fn calc_decision<'a>(
    vehicle: &mut VehicleComponent,
    map: &Map,
    time: &TimeInfo,
    trans: &Transform,
    self_obj: &PhysicsObject,
    neighs: impl Iterator<Item = (Vec2, &'a PhysicsObject)>,
) {
    vehicle.desired_speed = 0.0;

    if vehicle.wait_time > 0.0 {
        vehicle.wait_time -= time.delta;
        return;
    }
    let objective: Vec2 = unwrap_or!(vehicle.itinerary.get_point(), return);

    let is_terminal = vehicle.itinerary.is_terminal();

    let front_dist = calc_front_dist(vehicle, trans, self_obj, neighs);

    let position = trans.position();
    let speed = self_obj.speed;
    if speed.abs() < 0.2 && front_dist < 1.5 {
        vehicle.wait_time = (position.x * 1000.0).fract().abs() * 0.5;
        return;
    }

    let delta_pos: Vec2 = objective - position;
    let (dir_to_pos, dist_to_pos) = unwrap_or!(delta_pos.dir_dist(), return);

    let time_to_stop = speed / vehicle.kind.deceleration();
    let stop_dist = time_to_stop * speed / 2.0;

    vehicle.desired_dir = dir_to_pos;
    vehicle.desired_speed = vehicle.kind.cruising_speed();

    // Close to terminal objective
    if is_terminal && dist_to_pos < 1.0 + stop_dist {
        vehicle.desired_speed = 0.0;
    }

    if let Some(Traversable {
        kind: TraverseKind::Lane(l_id),
        ..
    }) = vehicle.itinerary.get_travers()
    {
        if let Some(l) = map.lanes().get(*l_id) {
            let dist_to_light = l.control_point().distance(position);
            match l.control.get_behavior(time.time_seconds) {
                TrafficBehavior::RED | TrafficBehavior::ORANGE => {
                    if dist_to_light
                        < OBJECTIVE_OK_DIST * 1.05
                            + 2.0
                            + stop_dist
                            + (vehicle.kind.width() / 2.0 - OBJECTIVE_OK_DIST).max(0.0)
                    {
                        vehicle.desired_speed = 0.0;
                    }
                }
                TrafficBehavior::STOP => {
                    if dist_to_light < OBJECTIVE_OK_DIST * 0.95 + stop_dist {
                        vehicle.desired_speed = 0.0;
                    }
                }
                _ => {}
            }
        }
    }

    // Not facing the objective
    if dir_to_pos.dot(trans.direction()) < 0.8 {
        vehicle.desired_speed = vehicle.desired_speed.min(6.0);
    }

    // Stop at 80 cm of object in front
    if front_dist < 0.8 + stop_dist {
        vehicle.desired_speed = 0.0;
    }
}

fn calc_front_dist<'a>(
    vehicle: &mut VehicleComponent,
    trans: &Transform,
    self_obj: &PhysicsObject,
    neighs: impl Iterator<Item = (Vec2, &'a PhysicsObject)>,
) -> f32 {
    let position = trans.position();
    let direction = trans.direction();

    let mut min_front_dist: f32 = 50.0;

    let my_ray = Ray {
        from: position - direction * vehicle.kind.width() / 2.0,
        dir: direction,
    };

    let my_radius = self_obj.radius;
    let speed = self_obj.speed;

    let on_lane = vehicle.itinerary.get_travers().unwrap().kind.is_lane();

    // Collision avoidance
    for (his_pos, nei_physics_obj) in neighs {
        // Ignore myself
        if std::ptr::eq(nei_physics_obj, self_obj) {
            continue;
        }

        let towards_vec: Vec2 = his_pos - position;
        let (towards_dir, dist) = unwrap_or!(towards_vec.dir_dist(), continue);

        // cos of angle from self to obj
        let cos_angle = towards_dir.dot(direction);

        // Ignore things behind
        if cos_angle < 0.0 {
            continue;
        }

        let dist_to_side = towards_vec.perp_dot(direction).abs();

        let is_vehicle = matches!(nei_physics_obj.group, PhysicsGroup::Vehicles);

        let cos_direction_angle = nei_physics_obj.dir.dot(direction);

        // front cone
        if cos_angle > 0.85 - 0.015 * speed.min(10.0)
            && (!is_vehicle || cos_direction_angle > 0.0)
            && (!on_lane || dist_to_side < 3.0)
        {
            let mut dist_to_obj = dist - my_radius - nei_physics_obj.radius;
            if !is_vehicle {
                dist_to_obj -= 1.0;
            }
            min_front_dist = min_front_dist.min(dist_to_obj);
            continue;
        }

        // don't do ray checks for other things than cars
        if !is_vehicle {
            continue;
        }

        // closest win
        let his_ray = Ray {
            from: his_pos - nei_physics_obj.radius * nei_physics_obj.dir,
            dir: nei_physics_obj.dir,
        };

        let (my_dist, his_dist) = unwrap_or!(both_dist_to_inter(my_ray, his_ray), continue);

        if my_dist - speed.min(2.5) - my_radius
            < his_dist - nei_physics_obj.speed.min(2.5) - nei_physics_obj.radius
        {
            continue;
        }

        min_front_dist = min_front_dist.min(dist - my_radius - nei_physics_obj.radius - 5.0);
    }
    min_front_dist
}
