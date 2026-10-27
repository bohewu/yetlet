//! Application-only preset and scheduler. No renderer or physics dependency.
pub const DT: f64 = 1.0 / 60.0;
pub const MAX_BODIES: u32 = 64;
pub const ROW_FLOATS: usize = 19;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scenario {
    #[default]
    Falling,
    Tower,
    Dominoes,
    Slope,
}
impl Scenario {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Falling => "falling",
            Self::Tower => "tower",
            Self::Dominoes => "dominoes",
            Self::Slope => "slope",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Falling => "Falling objects",
            Self::Tower => "Stack impact",
            Self::Dominoes => "Domino chain",
            Self::Slope => "Slope comparison",
        }
    }
}
impl std::str::FromStr for Scenario {
    type Err = &'static str;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "falling" => Ok(Self::Falling),
            "tower" => Ok(Self::Tower),
            "dominoes" => Ok(Self::Dominoes),
            "slope" => Ok(Self::Slope),
            _ => Err("invalid scenario"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parameters {
    pub scenario: Scenario,
    pub count: u32,
    pub gravity: f32,
    pub speed: f64,
}
impl Default for Parameters {
    fn default() -> Self {
        Self {
            scenario: Scenario::default(),
            count: 24,
            gravity: 9.81,
            speed: 1.0,
        }
    }
}
impl Parameters {
    pub fn validate(self) -> Result<Self, &'static str> {
        if !(1..=MAX_BODIES).contains(&self.count)
            || !self.gravity.is_finite()
            || !(0.0..=20.0).contains(&self.gravity)
            || ![0.25, 0.5, 1.0, 2.0].contains(&self.speed)
        {
            return Err("invalid bounded preset");
        }
        Ok(self)
    }
}
#[derive(Clone, Debug)]
pub struct Object {
    pub id: u32,
    pub sphere: bool,
    pub center: [f32; 3],
    pub size: [f32; 3],
    pub rotation: [f32; 4],
    pub color: [f32; 4],
    pub velocity: [f32; 3],
    pub friction: f32,
}
impl Object {
    pub fn floats(&self) -> [f32; ROW_FLOATS] {
        [
            self.id as f32,
            if self.sphere { 1.0 } else { 0.0 },
            self.center[0],
            self.center[1],
            self.center[2],
            self.rotation[0],
            self.rotation[1],
            self.rotation[2],
            self.rotation[3],
            self.size[0],
            self.size[1],
            self.size[2],
            self.color[0],
            self.color[1],
            self.color[2],
            self.color[3],
            self.velocity[0],
            self.velocity[1],
            self.velocity[2],
        ]
    }
}
pub fn preset(parameters: Parameters) -> Result<Vec<Object>, &'static str> {
    let p = parameters.validate()?;
    let mut objects = (0..p.count)
        .map(|i| Object {
            id: i + 1,
            sphere: i % 2 == 1,
            center: [
                (i % 8) as f32 * 2.0 - 7.0,
                1.0 + (i % 4) as f32 * 1.5,
                (i / 8) as f32 * 2.0 - 7.0,
            ],
            size: [0.8; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            color: if i % 2 == 1 {
                [0.98, 0.61, 0.22, 1.0]
            } else {
                [0.18, 0.70, 0.84, 1.0]
            },
            velocity: [0.0; 3],
            friction: 0.6,
        })
        .collect::<Vec<_>>();
    for (i, row) in objects.iter_mut().enumerate() {
        match p.scenario {
            Scenario::Falling => {}
            Scenario::Tower => {
                let blocks = if p.count > 1 { p.count - 1 } else { 1 };
                if i as u32 == blocks {
                    row.sphere = true;
                    row.center = [0.4, blocks.div_ceil(3) as f32 * 0.52 + 2.0, 0.0];
                    row.size = [1.1; 3];
                    row.velocity = [0.0, -0.8, 0.0];
                    row.color = [0.98, 0.61, 0.22, 1.0];
                } else {
                    let layer = i / 3;
                    let lane = (i % 3) as f32 - 1.0;
                    row.sphere = false;
                    row.size = [2.4, 0.5, 0.78];
                    row.center = if layer % 2 == 0 {
                        [0.0, 0.26 + layer as f32 * 0.52, lane * 0.82]
                    } else {
                        [lane * 0.82, 0.26 + layer as f32 * 0.52, 0.0]
                    };
                    if layer % 2 == 1 {
                        row.rotation = [
                            0.0,
                            std::f32::consts::FRAC_1_SQRT_2,
                            0.0,
                            std::f32::consts::FRAC_1_SQRT_2,
                        ];
                    }
                    row.color = [0.18, 0.70, 0.84, 1.0];
                }
            }
            Scenario::Dominoes => {
                let angle = -std::f32::consts::FRAC_PI_4 * 3.0 + i as f32 * 0.08;
                let yaw = -angle - std::f32::consts::FRAC_PI_2;
                let lean: f32 = if i == 0 { -0.28 } else { 0.0 };
                let (sy, cy) = (yaw * 0.5).sin_cos();
                let (sz, cz) = (lean * 0.5).sin_cos();
                row.sphere = false;
                row.center = [6.0 * angle.cos(), 0.72, 6.0 * angle.sin()];
                row.size = [0.16, 1.4, 0.75];
                row.rotation = [sy * sz, sy * cz, cy * sz, cy * cz];
                row.color = [0.18, 0.70, 0.84, 1.0];
                if i == 0 {
                    row.velocity = [-angle.sin() * 0.6, 0.0, angle.cos() * 0.6];
                    row.color = [0.98, 0.61, 0.22, 1.0];
                }
            }
            Scenario::Slope => {
                let angle = 25.0_f32.to_radians();
                let (sin, cos) = angle.sin_cos();
                let lane = i % 4;
                let along = -4.8 + (i / 4) as f32 * 0.6;
                row.sphere = lane % 2 == 1;
                row.size = [0.45; 3];
                row.center = [
                    along * cos + 0.495 * sin,
                    3.0 - along * sin + 0.495 * cos,
                    (lane as f32 - 1.5) * 3.0,
                ];
                row.rotation = if row.sphere {
                    [0.0, 0.0, 0.0, 1.0]
                } else {
                    [0.0, 0.0, (-angle * 0.5).sin(), (angle * 0.5).cos()]
                };
                row.friction = if lane < 2 { 0.1 } else { 0.9 };
                row.color = if lane < 2 {
                    [0.18, 0.70, 0.84, 1.0]
                } else {
                    [0.98, 0.61, 0.22, 1.0]
                };
                row.velocity = [0.0; 3];
            }
        }
    }
    for (i, (center, size)) in [
        ([0.0, -0.5, 0.0], [20.0, 1.0, 20.0]),
        ([-10.0, 4.5, 0.0], [0.5, 10.0, 20.0]),
        ([10.0, 4.5, 0.0], [0.5, 10.0, 20.0]),
        ([0.0, 4.5, -10.0], [20.0, 10.0, 0.5]),
        ([0.0, 4.5, 10.0], [20.0, 10.0, 0.5]),
    ]
    .into_iter()
    .enumerate()
    {
        objects.push(Object {
            id: 1001 + i as u32,
            sphere: false,
            center,
            size,
            rotation: [0.0, 0.0, 0.0, 1.0],
            color: [0.23, 0.29, 0.35, 1.0],
            velocity: [0.0; 3],
            friction: 0.6,
        });
    }
    if p.scenario == Scenario::Slope {
        let ramp = objects.iter_mut().find(|r| r.id == 1002).unwrap();
        let angle = -25.0_f32.to_radians();
        ramp.center = [0.0, 3.0, 0.0];
        ramp.size = [11.0, 0.5, 16.0];
        ramp.rotation = [0.0, 0.0, (angle * 0.5).sin(), (angle * 0.5).cos()];
        ramp.friction = 0.2;
        ramp.color = [0.36, 0.45, 0.49, 1.0];
    }
    Ok(objects)
}
#[derive(Default, Debug)]
pub struct Clock {
    pub running: bool,
    previous: Option<f64>,
    remainder: f64,
    pub dropped: f64,
}
impl Clock {
    pub fn start(&mut self, now: f64) -> Result<(), &'static str> {
        if !now.is_finite() || now < 0.0 {
            return Err("invalid monotonic timestamp");
        }
        self.running = true;
        self.previous = Some(now);
        self.remainder = 0.0;
        Ok(())
    }
    pub fn pause(&mut self) {
        self.running = false;
        self.previous = None;
        self.remainder = 0.0;
    }
    pub fn plan(&mut self, now: f64, speed: f64) -> Result<usize, &'static str> {
        if !self.running {
            return Ok(0);
        }
        let Some(previous) = self.previous else {
            self.pause();
            return Err("clock not started");
        };
        if !now.is_finite() || now < previous || ![0.25, 0.5, 1.0, 2.0].contains(&speed) {
            self.pause();
            return Err("invalid monotonic timestamp");
        }
        self.previous = Some(now);
        let delta = (now - previous) / 1000.0;
        self.dropped += (delta - 0.1).max(0.0) * speed;
        self.remainder += delta.min(0.1) * speed;
        let whole = (self.remainder / DT).floor() as usize;
        let steps = whole.min(4);
        self.remainder -= whole as f64 * DT;
        self.dropped += (whole - steps) as f64 * DT;
        if !self.dropped.is_finite() {
            self.pause();
            return Err("clock range exhausted");
        }
        Ok(steps)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn yt198_slope_bounds_and_normal_clearance() {
        assert_eq!("slope".parse::<Scenario>().unwrap(), Scenario::Slope);
        for count in [1, 4, 24, 64] {
            let rows = preset(Parameters {
                scenario: Scenario::Slope,
                count,
                ..Parameters::default()
            })
            .unwrap();
            assert_eq!(rows.len(), count as usize + 5);
            let ramp = rows.iter().find(|r| r.id == 1002).unwrap();
            let q = ramp.rotation;
            let normal = [-2.0 * q[2] * q[3], 1.0 - 2.0 * q[2] * q[2], 0.0];
            assert_eq!(ramp.friction, 0.2);
            for (i, row) in rows[..count as usize].iter().enumerate() {
                assert_eq!(row.id, i as u32 + 1);
                assert_eq!(row.floats().len(), 19);
                assert!(row.floats().iter().all(|n| n.is_finite()));
                assert!(
                    row.center[0].abs() < 8.0
                        && row.center[2].abs() < 8.0
                        && row.center[1] > 0.0
                        && row.center[1] < 8.0
                );
                assert!((row.rotation.iter().map(|x| x * x).sum::<f32>() - 1.0).abs() < 1e-5);
                let clearance = row
                    .center
                    .iter()
                    .zip(ramp.center)
                    .zip(normal)
                    .map(|((x, c), n)| (x - c) * n)
                    .sum::<f32>();
                assert!(
                    clearance > 0.25 + 0.225,
                    "initial body must clear the ramp: {clearance}"
                );
                assert_eq!(row.sphere, i % 2 == 1);
                assert_eq!(row.friction, if i % 4 < 2 { 0.1 } else { 0.9 });
                assert_eq!(row.velocity, [0.0; 3]);
            }
            assert_eq!(rows.iter().filter(|r| r.id >= 1000).count(), 5);
        }
    }
    #[test]
    fn yt197_scene_names_and_bounded_new_presets() {
        assert_eq!(Scenario::default(), Scenario::Falling);
        for name in ["falling", "tower", "dominoes"] {
            assert_eq!(name.parse::<Scenario>().unwrap().as_str(), name);
        }
        assert!("unknown".parse::<Scenario>().is_err());
        for scenario in [Scenario::Tower, Scenario::Dominoes] {
            for count in [1, 24, 64] {
                let rows = preset(Parameters {
                    scenario,
                    count,
                    ..Parameters::default()
                })
                .unwrap();
                assert_eq!(rows.len(), count as usize + 5);
                for (i, row) in rows[..count as usize].iter().enumerate() {
                    assert_eq!(row.id, i as u32 + 1);
                    assert!(row.floats().iter().all(|n| n.is_finite()));
                    assert!(row.center[0].abs() < 8.0 && row.center[2].abs() < 8.0);
                    assert!(row.center[1] > 0.0 && row.center[1] < 16.0);
                    assert!((row.rotation.iter().map(|n| n * n).sum::<f32>() - 1.0).abs() < 1e-5);
                    assert!(row.size.iter().all(|n| *n > 0.0));
                }
                assert_eq!(
                    rows[..count as usize].iter().filter(|r| r.sphere).count(),
                    usize::from(scenario == Scenario::Tower && count > 1)
                );
                assert_eq!(rows[count as usize].id, 1001);
                assert!(
                    preset(Parameters {
                        scenario,
                        count: 65,
                        ..Parameters::default()
                    })
                    .is_err()
                );
            }
        }
    }
    #[test]
    fn bounded_presets_reject_without_partial_output() {
        assert_eq!(preset(Parameters::default()).unwrap().len(), 29);
        assert_eq!(
            preset(Parameters {
                count: 64,
                ..Parameters::default()
            })
            .unwrap()
            .len(),
            69
        );
        for p in [
            Parameters {
                count: 0,
                ..Parameters::default()
            },
            Parameters {
                count: 65,
                ..Parameters::default()
            },
            Parameters {
                gravity: f32::NAN,
                ..Parameters::default()
            },
            Parameters {
                speed: 3.0,
                ..Parameters::default()
            },
        ] {
            assert!(preset(p).is_err());
        }
        let p = preset(Parameters {
            count: 64,
            ..Parameters::default()
        })
        .unwrap();
        for (i, a) in p[..64].iter().enumerate() {
            for b in &p[i + 1..64] {
                assert!(
                    a.center
                        .iter()
                        .zip(b.center)
                        .any(|(x, y)| (*x - y).abs() >= 0.8)
                );
            }
        }
    }
    #[test]
    fn fixed_step_debt_pause_and_clock_refusal() {
        let mut c = Clock::default();
        assert_eq!(c.plan(100.0, 1.0).unwrap(), 0);
        c.start(0.0).unwrap();
        assert_eq!(c.plan(1000.0, 2.0).unwrap(), 4);
        assert!((c.dropped - (2.0 - 4.0 * DT)).abs() < 1e-10);
        assert!(c.remainder < DT);
        c.pause();
        c.start(2000.0).unwrap();
        assert_eq!(c.plan(2001.0, 1.0).unwrap(), 0);
        assert!(c.plan(1999.0, 1.0).is_err());
        assert!(!c.running);
    }
}
