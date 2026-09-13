//! M0 Bootstrap smoke test (docs/GDD.md §5.8): prove that nannou (rendering)
//! and rapier2d (physics) load and run together, at 60fps, with a box
//! falling onto a floor. The real Sim / EnergyGraph / Part architecture
//! (mirroring OpenTIM's `part.rs` / `level_file_format.rs` / `atmosphere.rs`
//! shape — see CLAUDE.md) arrives in M1+. This file is intentionally a
//! throwaway wiring check, not the sim core.

use nannou::prelude::*;
use rapier2d::prelude::*;

const PIXELS_PER_METER: f32 = 32.0;
const FIXED_DT: f32 = 1.0 / 120.0;
const MAX_SUBSTEPS_PER_FRAME: u32 = 4;

struct Physics {
    gravity: Vector,
    integration_parameters: IntegrationParameters,
    pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
}

impl Physics {
    fn step(&mut self) {
        self.pipeline.step(
            self.gravity,
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            &(),
            &(),
        );
    }
}

struct Model {
    physics: Physics,
    ground: RigidBodyHandle,
    ground_half_extents: Vector,
    falling_box: RigidBodyHandle,
    box_half_extent: f32,
    accumulator: f32,
    recent_frame_seconds: Vec<f32>,
    seconds_since_fps_log: f32,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(900, 700)
        .title("ChainWorks — M0 Bootstrap")
        .view(view)
        .build()
        .unwrap();

    let mut integration_parameters = IntegrationParameters::default();
    integration_parameters.dt = FIXED_DT;

    let mut bodies = RigidBodySet::new();
    let mut colliders = ColliderSet::new();

    // Ground: a wide fixed slab. nannou uses a standard +y-up screen, so
    // "down" is negative y here (unlike the +y-down convention the earlier
    // Pixi-based draft assumed — see docs/GDD.md update note).
    let ground_half_extents = Vector::new(20.0, 0.5);
    let ground = bodies.insert(RigidBodyBuilder::fixed().translation(Vector::new(0.0, -6.0)));
    colliders.insert_with_parent(
        ColliderBuilder::cuboid(ground_half_extents.x, ground_half_extents.y).friction(0.6),
        ground,
        &mut bodies,
    );

    let box_half_extent = 0.5;
    let falling_box =
        bodies.insert(RigidBodyBuilder::dynamic().translation(Vector::new(0.0, 6.0)));
    colliders.insert_with_parent(
        ColliderBuilder::cuboid(box_half_extent, box_half_extent)
            .restitution(0.25)
            .friction(0.5),
        falling_box,
        &mut bodies,
    );

    let physics = Physics {
        gravity: Vector::new(0.0, -9.81),
        integration_parameters,
        pipeline: PhysicsPipeline::new(),
        islands: IslandManager::new(),
        broad_phase: BroadPhaseBvh::new(),
        narrow_phase: NarrowPhase::new(),
        bodies,
        colliders,
        impulse_joints: ImpulseJointSet::new(),
        multibody_joints: MultibodyJointSet::new(),
        ccd_solver: CCDSolver::new(),
    };

    Model {
        physics,
        ground,
        ground_half_extents,
        falling_box,
        box_half_extent,
        accumulator: 0.0,
        recent_frame_seconds: Vec::with_capacity(120),
        seconds_since_fps_log: 0.0,
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let dt_seconds = update.since_last.as_secs_f32().min(0.1);
    model.accumulator += dt_seconds;

    let mut steps = 0;
    while model.accumulator >= FIXED_DT && steps < MAX_SUBSTEPS_PER_FRAME {
        model.physics.step();
        model.accumulator -= FIXED_DT;
        steps += 1;
    }
    // Behind schedule — drop the remainder rather than spiral or break determinism.
    if steps == MAX_SUBSTEPS_PER_FRAME {
        model.accumulator = 0.0;
    }

    if dt_seconds > 0.0 {
        model.recent_frame_seconds.push(dt_seconds);
        if model.recent_frame_seconds.len() > 120 {
            model.recent_frame_seconds.remove(0);
        }
    }

    model.seconds_since_fps_log += dt_seconds;
    if model.seconds_since_fps_log >= 1.0 {
        let average_frame_seconds =
            model.recent_frame_seconds.iter().sum::<f32>() / model.recent_frame_seconds.len() as f32;
        let box_y = model.physics.bodies[model.falling_box].translation().y;
        println!(
            "fps: {:.0}  box.y: {:.3}m",
            1.0 / average_frame_seconds,
            box_y
        );
        model.seconds_since_fps_log = 0.0;
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(rgb8(0x14, 0x17, 0x1f));

    let ground_pos = model.physics.bodies[model.ground].translation();
    draw.rect()
        .x_y(
            ground_pos.x * PIXELS_PER_METER,
            ground_pos.y * PIXELS_PER_METER,
        )
        .w_h(
            model.ground_half_extents.x * 2.0 * PIXELS_PER_METER,
            model.ground_half_extents.y * 2.0 * PIXELS_PER_METER,
        )
        .rgb8(0x2a, 0x30, 0x40);

    let box_body = &model.physics.bodies[model.falling_box];
    let box_pos = box_body.translation();
    let box_angle = box_body.rotation().angle();
    draw.rect()
        .x_y(box_pos.x * PIXELS_PER_METER, box_pos.y * PIXELS_PER_METER)
        .w_h(
            model.box_half_extent * 2.0 * PIXELS_PER_METER,
            model.box_half_extent * 2.0 * PIXELS_PER_METER,
        )
        .rotate(box_angle)
        .rgb8(0x6c, 0x8c, 0xff);

    draw.to_frame(app, &frame).unwrap();
}

fn main() {
    nannou::app(model).update(update).run();
}
