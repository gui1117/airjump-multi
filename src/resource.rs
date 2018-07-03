pub struct UpdateTime(pub f32);

#[derive(Deref, DerefMut)]
pub struct DrawImage(pub Option<::Image>);

pub type PhysicWorld = ::nphysics2d::world::World<f32>;

#[derive(Deref, DerefMut)]
pub struct BodiesMap(::fnv::FnvHashMap<::nphysics2d::object::BodyHandle, ::specs::Entity>);

impl BodiesMap {
    pub fn new() -> Self {
        BodiesMap(::fnv::FnvHashMap::default())
    }
}
