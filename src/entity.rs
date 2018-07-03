use specs;
use ncollide2d;
use nphysics2d;
use nphysics2d::volumetric::Volumetric;
use specs::Builder;

const BALL_RADIUS: f32 = 1.0;
const BALL_DENSITY: f32 = 1.0;
pub const BALL_VELOCITY: f32 = 1.0;

const GONG_RADIUS: f32 = 1.0;
const GONG_DENSITY: f32 = 1.0;
const GONG_JOINT_LIMIT: f32 = 1.0;
const GONG_POSITION_Y: f32 = 10.0;

const GROUND_POSITION_Y: f32 = -10.0;
const GROUND_FLAT_LENGTH: f32 = 10.0;
const GROUND_BORDER_ANGLE: f32 = 1.0;
const GROUND_BORDER_LENGTH: f32 = 100.0;

pub fn create_ball(gamepad_id: usize, gamepad_parts: [bool; 2], world: &mut specs::World) {
    let entity = world.create_entity()
        .with(::component::Image(BALL_RADIUS, ::Image::Ball))
        .with(::component::Control { 
            gamepad_id,
            parts: gamepad_parts,
        })
        .with(::component::Airjump(false))
        .with(::component::Contactor(vec![]))
        .build();

    let mut physic_world = world.write_resource::<::resource::PhysicWorld>();

    let shape = ncollide2d::shape::ShapeHandle::new(ncollide2d::shape::Ball::new(BALL_RADIUS));

    let body_handle = ::component::RigidBody::safe_insert(
        entity,
        ::na::Isometry2::new(::na::Vector2::new(0.0, 0.0), 0.0),
        shape.inertia(BALL_DENSITY),
        shape.center_of_mass(),
        nphysics2d::object::BodyStatus::Dynamic,
        &mut world.write_storage(),
        &mut physic_world,
        &mut world.write_resource(),
    );

    physic_world.add_collider(
        0.0,
        shape,
        body_handle.0,
        ::na::one(),
        nphysics2d::object::Material::new(0.0, 0.0),
    );
}

pub fn create_gong(world: &mut specs::World) {
    let entity = world.create_entity()
        .with(::component::Image(GONG_RADIUS, ::Image::Ball))
        .build();

    let position = ::na::Point2::new(0.0, GONG_POSITION_Y);

    let mut physic_world = world.write_resource::<::resource::PhysicWorld>();

    let shape = ncollide2d::shape::ShapeHandle::new(ncollide2d::shape::Ball::new(GONG_RADIUS));

    let body_handle = ::component::RigidBody::safe_insert(
        entity,
        ::na::Isometry2::new(position.coords, 0.0),
        shape.inertia(GONG_DENSITY),
        shape.center_of_mass(),
        nphysics2d::object::BodyStatus::Dynamic,
        &mut world.write_storage(),
        &mut physic_world,
        &mut world.write_resource(),
    );

    physic_world.add_constraint(
        nphysics2d::joint::MouseConstraint::new(
            nphysics2d::object::BodyHandle::ground(),
            body_handle.0,
            position,
            nphysics2d::math::Point::new(0.0, 0.0),
            GONG_JOINT_LIMIT,
        )
    );

    physic_world.add_collider(
        0.0,
        shape,
        body_handle.0,
        ::na::one(),
        nphysics2d::object::Material::new(0.0, 0.0),
    );
}

pub fn create_ground(world: &mut specs::World) {
    let entity = world.create_entity()
        .with(::component::AirjumpRestorer)
        .build();

    let mut physic_world = world.write_resource::<::resource::PhysicWorld>();
    let mut bodies_map = world.write_resource::<::resource::BodiesMap>();
    if bodies_map.contains_key(&::nphysics2d::object::BodyHandle::ground()) {
        panic!("Only one ground can be inserted into world");
    }
    bodies_map.insert(::nphysics2d::object::BodyHandle::ground(), entity);

    let shape = ncollide2d::shape::ShapeHandle::new(
        ncollide2d::shape::Polyline::new(vec![
            ::na::Point2::new(-GROUND_FLAT_LENGTH / 2.0 - GROUND_BORDER_LENGTH*GROUND_BORDER_ANGLE.cos(), GROUND_POSITION_Y + GROUND_BORDER_LENGTH*GROUND_BORDER_ANGLE.sin()),
            ::na::Point2::new(-GROUND_FLAT_LENGTH / 2.0, GROUND_POSITION_Y),
            ::na::Point2::new(GROUND_FLAT_LENGTH / 2.0, GROUND_POSITION_Y),
            ::na::Point2::new(GROUND_FLAT_LENGTH / 2.0 + GROUND_BORDER_LENGTH*GROUND_BORDER_ANGLE.cos(), GROUND_POSITION_Y + GROUND_BORDER_LENGTH*GROUND_BORDER_ANGLE.sin()),
        ])
    );

    physic_world.add_collider(
        0.0,
        shape,
        nphysics2d::object::BodyHandle::ground(),
        ::na::one(),
        nphysics2d::object::Material::new(0.0, 0.0),
    );
}
