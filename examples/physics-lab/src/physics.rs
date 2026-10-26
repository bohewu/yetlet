//! Private application physics. Compiled only by this example's opt-in feature.
use crate::model::{Clock, DT, Object, Parameters, preset};
use rapier3d::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Simulation {
    world: PhysicsWorld,
    objects: Vec<(Object, RigidBodyHandle, ColliderHandle)>,
    snapshot: Vec<Object>,
    parameters: Parameters,
    clock: Clock,
    ticks: u32,
    valid: bool,
}
impl Simulation {
    pub fn create(parameters: Parameters) -> Result<Self, &'static str> {
        let snapshot = preset(parameters)?;
        let mut world = PhysicsWorld::default();
        world.gravity = Vector::new(0.0, -parameters.gravity, 0.0);
        world.integration_parameters.dt = DT as f32;
        world.integration_parameters.max_ccd_substeps = 1;
        let mut objects = Vec::new();
        for row in &snapshot {
            let dynamic = row.id < 1000;
            let body = if dynamic {
                RigidBodyBuilder::dynamic().ccd_enabled(true)
            } else {
                RigidBodyBuilder::fixed()
            };
            let mut pose = Pose::IDENTITY;
            pose.translation = Vector::from_array(row.center);
            pose.rotation = Rotation::from_xyzw(
                row.rotation[0],
                row.rotation[1],
                row.rotation[2],
                row.rotation[3],
            );
            let body = body.pose(pose).linvel(Vector::from_array(row.velocity));
            let collider = if row.sphere {
                ColliderBuilder::ball(row.size[0] / 2.0)
            } else {
                ColliderBuilder::cuboid(row.size[0] / 2.0, row.size[1] / 2.0, row.size[2] / 2.0)
            };
            let collider = if dynamic {
                collider.mass(1.0)
            } else {
                collider
            };
            let collider = collider.friction(row.friction).restitution(0.25);
            let collider = if parameters.scenario == crate::model::Scenario::Slope {
                collider.friction_combine_rule(CoefficientCombineRule::Average)
            } else {
                collider
            };
            let (b, c) = world.insert(body, collider);
            objects.push((row.clone(), b, c));
        }
        Ok(Self {
            world,
            objects,
            snapshot,
            parameters,
            clock: Clock::default(),
            ticks: 0,
            valid: true,
        })
    }
    fn tick(&mut self) -> Result<(), &'static str> {
        if !self.valid {
            return Err("invalid-simulation; reset required");
        }
        let Some(ticks) = self.ticks.checked_add(1) else {
            self.valid = false;
            self.clock.pause();
            return Err("tick range exhausted");
        };
        self.world.step();
        let mut next = Vec::with_capacity(self.objects.len());
        for (record, handle, _) in &self.objects {
            let body = &self.world.bodies[*handle];
            let mut row = record.clone();
            row.center = body.translation().to_array();
            row.rotation = body.rotation().to_array();
            row.velocity = body.linvel().to_array();
            if row.center.iter().any(|x| !x.is_finite() || x.abs() > 100.0)
                || row
                    .rotation
                    .iter()
                    .chain(&row.velocity)
                    .any(|x| !x.is_finite())
            {
                self.valid = false;
                self.clock.pause();
                return Err("invalid-simulation; reset required");
            }
            next.push(row);
        }
        self.snapshot = next;
        self.ticks = ticks;
        Ok(())
    }
    pub fn start_at(&mut self, now: f64) -> Result<(), &'static str> {
        if !self.valid {
            return Err("invalid-simulation; reset required");
        }
        self.clock.start(now)
    }
    pub fn advance_at(&mut self, now: f64) -> Result<usize, &'static str> {
        let n = self.clock.plan(now, self.parameters.speed)?;
        for _ in 0..n {
            self.tick()?;
        }
        Ok(n)
    }
    pub fn step_once(&mut self) -> Result<(), &'static str> {
        if self.clock.running {
            return Err("pause before stepping");
        }
        self.tick()
    }
    pub fn replace(&mut self, parameters: Parameters) -> Result<(), &'static str> {
        if self.clock.running {
            return Err("pause before reset");
        }
        let next = Self::create(parameters)?;
        *self = next;
        Ok(())
    }
    pub fn rows(&self) -> &[Object] {
        &self.snapshot
    }
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Simulation {
    pub fn new(count: u32, gravity: f32, speed: f64, scenario: &str) -> Result<Simulation, String> {
        Self::create(Parameters {
            scenario: scenario.parse().map_err(str::to_owned)?,
            count,
            gravity,
            speed,
        })
        .map_err(str::to_owned)
    }
    pub fn resume(&mut self, now: f64) -> Result<(), String> {
        self.start_at(now).map_err(str::to_owned)
    }
    pub fn advance(&mut self, now: f64) -> Result<u32, String> {
        self.advance_at(now)
            .map(|n| n as u32)
            .map_err(str::to_owned)
    }
    pub fn pause(&mut self) {
        self.clock.pause();
    }
    pub fn step(&mut self) -> Result<(), String> {
        self.step_once().map_err(str::to_owned)
    }
    pub fn reset(
        &mut self,
        count: u32,
        gravity: f32,
        speed: f64,
        scenario: &str,
    ) -> Result<(), String> {
        self.replace(Parameters {
            scenario: scenario.parse().map_err(str::to_owned)?,
            count,
            gravity,
            speed,
        })
        .map_err(str::to_owned)
    }
    pub fn poses(&self) -> Vec<f32> {
        self.snapshot.iter().flat_map(Object::floats).collect()
    }
    pub fn ticks(&self) -> u32 {
        self.ticks
    }
    pub fn dropped_seconds(&self) -> f64 {
        self.clock.dropped
    }
    pub fn running(&self) -> bool {
        self.clock.running
    }
    pub fn valid(&self) -> bool {
        self.valid
    }
    pub fn pick(&self, ox: f32, oy: f32, oz: f32, dx: f32, dy: f32, dz: f32) -> u32 {
        if self.clock.running
            || !self.valid
            || [ox, oy, oz, dx, dy, dz].iter().any(|x| !x.is_finite())
        {
            return 0;
        }
        let direction = Vector::new(dx, dy, dz);
        if direction.length_squared() < 0.5 {
            return 0;
        }
        let ray = Ray::new(Vector::new(ox, oy, oz), direction.normalize());
        self.objects
            .iter()
            .filter_map(|(r, _, h)| {
                let c = &self.world.colliders[*h];
                c.shape()
                    .cast_ray(c.position(), &ray, 200.0, true)
                    .map(|t| (r.id, t))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| if id < 1000 { id } else { 0 })
            .unwrap_or(0)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Scenario;
    #[test]
    fn yt198_slope_contact_roll_slide_and_friction() {
        let mut sim = Simulation::new(4, 9.81, 1.0, "slope").unwrap();
        let initial = sim.rows().to_vec();
        for (record, _, handle) in &sim.objects {
            let collider = &sim.world.colliders[*handle];
            assert_eq!(collider.friction(), record.friction);
            assert_eq!(
                collider.friction_combine_rule(),
                CoefficientCombineRule::Average
            );
        }
        for _ in 0..60 {
            sim.step_once().unwrap();
        }
        assert!(sim.valid());
        let angle = 25.0_f32.to_radians();
        let along = |i: usize| {
            (sim.rows()[i].center[0] - initial[i].center[0]) * angle.cos()
                - (sim.rows()[i].center[1] - initial[i].center[1]) * angle.sin()
        };
        assert!(along(0) > 0.5, "lower-friction box slides: {}", along(0));
        assert!(
            along(2).abs() < 0.15,
            "higher-friction box holds: {}",
            along(2)
        );
        for i in [1, 3] {
            assert!(along(i) > 0.5, "sphere moves downhill: {}", along(i));
            assert!(
                sim.world.bodies[sim.objects[i].1].angvel().length() > 1.0,
                "sphere rotates rather than only translating"
            );
        }
    }
    #[test]
    fn yt197_initial_poses_and_atomic_scenario_reset() {
        let mut sim = Simulation::new(24, 9.81, 1.0, "dominoes").unwrap();
        for (record, handle, _) in &sim.objects {
            let body = &sim.world.bodies[*handle];
            assert_eq!(body.translation().to_array(), record.center);
            assert_eq!(body.linvel().to_array(), record.velocity);
            for (actual, expected) in body.rotation().to_array().iter().zip(record.rotation) {
                assert!((actual - expected).abs() < 1e-5);
            }
        }
        sim.step_once().unwrap();
        let poses = sim.poses();
        let ticks = sim.ticks();
        assert!(sim.reset(24, 9.81, 1.0, "unknown").is_err());
        assert_eq!(sim.poses(), poses);
        assert_eq!(sim.ticks(), ticks);
        sim.reset(12, 9.81, 1.0, "tower").unwrap();
        assert_eq!(sim.parameters.scenario, Scenario::Tower);
        assert_eq!(sim.rows().len(), 17);
        assert_eq!(sim.ticks(), 0);
    }
    #[test]
    fn yt197_tower_impact_and_domino_chain_progress() {
        let mut tower = Simulation::new(24, 9.81, 1.0, "tower").unwrap();
        let before = tower.rows().to_vec();
        for _ in 0..240 {
            tower.step_once().unwrap();
        }
        assert!(tower.valid());
        assert!(tower.rows()[23].center[1] < before[23].center[1] - 3.0);
        assert!(
            tower.rows()[..23].iter().zip(&before).any(|(a, b)| a
                .center
                .iter()
                .zip(b.center)
                .any(|(x, y)| (x - y).abs() > 0.1)),
            "striker must disturb the stack"
        );
        let mut dominoes = Simulation::new(24, 9.81, 1.0, "dominoes").unwrap();
        for _ in 0..600 {
            dominoes.step_once().unwrap();
        }
        assert!(dominoes.valid());
        let fallen = dominoes.rows()[..24]
            .iter()
            .filter(|r| {
                1.0 - 2.0 * (r.rotation[0] * r.rotation[0] + r.rotation[2] * r.rotation[2]) < 0.7
            })
            .count();
        assert!(
            fallen > 5,
            "chain must propagate beyond the first body: {fallen}"
        );
    }
    #[test]
    fn static_enclosure_occludes_hidden_objects_but_clear_ray_selects() {
        let sim = Simulation::create(Parameters {
            count: 1,
            ..Parameters::default()
        })
        .unwrap();
        // Camera outside the front wall: the wall is the nearest visible surface.
        assert_eq!(sim.pick(-7.0, 1.0, 20.0, 0.0, 0.0, -1.0), 0);
        // Looking down inside the enclosure hits the dynamic cube before the floor.
        assert_eq!(sim.pick(-7.0, 5.0, -7.0, 0.0, -1.0, 0.0), 1);
    }

    #[test]
    fn gravity_contact_pause_step_and_atomic_reset() {
        let mut sim = Simulation::create(Parameters {
            count: 1,
            ..Parameters::default()
        })
        .unwrap();
        let y = sim.rows()[0].center[1];
        sim.step_once().unwrap();
        assert!(sim.rows()[0].velocity[1] < 0.0);
        assert!(sim.rows()[0].center[1] < y);
        for _ in 0..239 {
            sim.step_once().unwrap();
        }
        let row = &sim.rows()[0];
        assert!(
            row.center[1] > 0.35 && row.center[1] < 0.48,
            "floor contact: {:?}",
            row.center
        );
        let ticks = sim.ticks();
        let poses = sim.poses();
        assert!(
            sim.replace(Parameters {
                count: 65,
                ..Parameters::default()
            })
            .is_err()
        );
        assert_eq!(sim.ticks(), ticks);
        assert_eq!(sim.poses(), poses);
        sim.start_at(0.0).unwrap();
        assert!(sim.step_once().is_err());
        assert_eq!(sim.advance_at(1000.0).unwrap(), 4);
        sim.pause();
        sim.replace(Parameters {
            count: 64,
            gravity: 0.0,
            speed: 2.0,
            ..Parameters::default()
        })
        .unwrap();
        assert_eq!(sim.rows().len(), 69);
        assert_eq!(sim.ticks(), 0);
        let before = sim.poses();
        sim.step_once().unwrap();
        assert_eq!(sim.poses(), before);
    }
    #[test]
    fn invalid_world_freezes_snapshot_and_requires_reset() {
        let mut s = Simulation::create(Parameters::default()).unwrap();
        let before = s.poses();
        let h = s.objects[0].1;
        s.world.bodies[h].set_translation(Vector::new(101.0, 1.0, 0.0), true);
        assert!(s.step_once().is_err());
        assert!(!s.valid());
        assert_eq!(s.poses(), before);
        assert!(s.start_at(0.0).is_err());
        s.replace(Parameters::default()).unwrap();
        assert!(s.valid());
    }
    #[test]
    fn collider_pick_is_local_and_refused_while_running() {
        let mut s = Simulation::create(Parameters {
            count: 1,
            ..Parameters::default()
        })
        .unwrap();
        assert_eq!(s.pick(-7.0, 5.0, -7.0, 0.0, -1.0, 0.0), 1);
        s.start_at(0.0).unwrap();
        assert_eq!(s.pick(-7.0, 5.0, -7.0, 0.0, -1.0, 0.0), 0);
    }
}
