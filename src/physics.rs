use rapier2d::prelude::*;

pub struct PhysicsSystem {
    gravity: Vector<Real>,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    joint_set: ImpulseJointSet,
    ccd_solver: CCDSolver,
    integration_parameters: IntegrationParameters,
}

impl PhysicsSystem {
    pub fn new() -> Self {
        Self {
            gravity: vector![0.0, -9.81],
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            joint_set: ImpulseJointSet::new(),
            ccd_solver: CCDSolver::new(),
            integration_parameters: IntegrationParameters::default(),
        }
    }
    
    pub fn update(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.joint_set,
            &mut MultibodyJointSet::new(),
            &mut self.ccd_solver,
            None,
            &(),
            &(),
        );
    }
}