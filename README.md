# Airjump Multi

This is a basic game showing a way to structure your code with `specs`, `nphysics` (and `vulkano`).

Each users control a ball that have one airjump, this airjump is restored on contact with the ground and other balls. The goal is to touch the gong.

This will explain some concept about the implementation. The most interesting part is integrate nphysics with specs.

Also this repository contains an appveyor script that uses Visual Studio 2013 which is useful to build for old versions of windows.

## Entity/Component/System (ECS)

ECS is a common pattern used in games.
Entities are sets of components, Components are structure that holds data and Systems are function that iterate on components to update their data.

In this game there are:
* Components:
  * `Airjump(bool)`: whether an airjump is available
  * `AirjumpRestorer`: a flag telling airjump system that a collision with this entity must restore others airjumps
  * `Control(gamepad_id, part)`: store which part of which gamepad controls the entity
  * `Image(imge)`: store the image to be drawn for the entity
  * `RigidBody(handle)`: store a handle of a rigid body in nphysics world
  * `Contactors(contacts)`: store entities in contact

* Systems:
  * `Physic`: update physic world and contactors
  * `Airjump`: uses `AirjumpRestorer`, `Contactors` components and modifies `Airjump`:
    ```rust
    // iterate on entities that contains airjump and contactor
    for (airjump, contactor) in (&mut airjumps, &contactors).join() {
        for entity in contactor.iter() {
            // if on contact with a restorer entity
            if restorers.get(*entity).is_some() {
                // then set airjump available
                airjump.0 = true;
            }
        }
    }
    ```
    * `Controller`: uses gamepad events and `Airjump` component and modifies `RigidBody` position and velocity.

* Entities:
  * ball: `Image`, `Control`, `Airjump`, `AirjumpRestorer`, `Contactor`, `RigidBody`
  * gong: `Image`, `RigidBody`
  * walls: `RigidBody`
  * ground: `AirjumpRestorer`, `RigidBody`

## Integrate nphysics with specs

This is probably the most interesting part.

Adding a rigid body to the nphysics world returns a handle, this handle can be used to borrow the actual rigid body by borrowing the nphysics world.

The handle is stored in a specs component storage and each system that want to use it must also use the nphysic world resource to actually access the data.
In order to be able to get the entity corresponding to a rigid body (while raycasting in the physic world for instance), I created a resource BodiesMap that map each body to an entity.
The main issue is how to make nphysics world coherent with handle specs component storage and bodies mapping.

* Enforce handles stored in specs to actually correspond to an existing nphysics body:

  I simply made the component buildable only from method that insert the body in nphysics world and bodies map at the same time as in the specs storage.
  ```rust
  pub fn safe_insert<'a>(
      entity: ::specs::Entity,
      // position, inertia, ...
      bodies_handle: &mut ::specs::WriteStorage<'a, ::component::RigidBody>,
      physic_world: &mut ::resource::PhysicWorld,
      bodies_map: &mut ::resource::BodiesMap,
  ) -> Self {
      let body_handle = physic_world.add_rigid_body(position, inertia ...);
      bodies_map.insert(body_handle, entity);
      bodies_handle.insert(entity, RigidBody(body_handle));
      RigidBody(body_handle)
  }
  ```

* Easily get the actual data from the handle by borrowing nphysics world:
  ```rust
  pub fn get<'a>(&'a self, physic_world: &'a PhysicWorld) -> &'a RigidBody {
      physic_world
          .rigid_body(self.0)
          .expect("Rigid body in specs does not exist in physic world")
  }

  pub fn get_mut<'a>( ... idem
  ```

* Enforce deletion of entities with a rigid body component to delete body in nphysics world:

  Here is the real issue: because deleting an entity with a rigid body component deletes only the handle and let the body in nphysics world

  A way to solve it could be to create a method safe_delete(entity, &rigid_body_handle_component, &mut physic_world, &mut bodies_map) that check if the entity has a
  rigid body handle and if so removes it from the physic world. But this is not handy at all.

  A better way is to use a specific storage for body handles that keeps track of removed components. Then we can regularly take the pendings removed handles and remove the corresponding body in nphysics world and bodies map.
  Pseudo code:
  ```rust
  pub struct RigidBody(BodyHandle);
  impl ::specs::Component for RigidBody {
      type Storage = RetainedStorage<Self>;
  }

  pub fn safe_maintain(world: &mut specs::World) {
      world.maintain();
      let mut physic_world = world.write_resource::<::resource::PhysicWorld>();
      let mut bodies_map = world.write_resource::<::resource::BodiesMap>();

      let retained = world.write_storage::<::component::RigidBody>().get_pending_handles();
      physic_world.remove_bodies(&retained);
      bodies_map.remove_bodies(&retained);
  }
  ```

See the actual implementation in [src/retained_storage.rs](), [src/component.rs]() and [src/main.rs]()

## Game state

In order to process gamepad inputs differently corresponding on the state of the game (menus or in-game) I used following trait object:

```rust
pub trait GameState {
    fn update(self: Box<Self>, world: &mut specs::World) -> Box<GameState>;
    fn event(self: Box<Self>, event: gilrs::ev::Event, world: &mut specs::World)
        -> Box<GameState>;
    fn gamepad(
        self: Box<Self>,
        id: usize,
        gamepad: &::gilrs::Gamepad,
        world: &mut specs::World,
    ) -> Box<GameState>;
    fn quit(&self) -> bool {
        false
    }
    fn paused(&self) -> bool;
}
```

Thus each game state can return self or the next state after each update, or events.

## Vulkano

There is not much to say here. It shows how to use abstract type of vulkano:

```rust
pub struct Graphics {
    surface: Arc<vulkano::swapchain::Surface<winit::Window>>,
    device: Arc<vulkano::device::Device>,
    queue: Arc<vulkano::device::Queue>,
    recreate_swapchain: bool,
    previous_frame_end: Box<vulkano::sync::GpuFuture>,
    swapchain: Arc<vulkano::swapchain::Swapchain<winit::Window>>,
    framebuffers: Option<Vec<Arc<vulkano::framebuffer::FramebufferAbstract + Sync + Send>>>,
    renderpass: Arc<vulkano::framebuffer::RenderPassAbstract + Sync + Send>,
    images: Vec<Arc<vulkano::image::SwapchainImage<winit::Window>>>,
    pipeline: Arc<vulkano::pipeline::GraphicsPipelineAbstract + Sync + Send>,
    vertex_buffer: Arc<vulkano::buffer::CpuAccessibleBuffer<[Vertex]>>,
    transform_buffer_pool: vulkano::buffer::CpuBufferPool<vs::ty::Transform>,
    view_buffer_pool: vulkano::buffer::CpuBufferPool<vs::ty::View>,
    textures: HashMap<::Image, (u32, u32, Arc<vulkano::image::ImageViewAccess + Sync + Send>)>,
    sets_pool: vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool<Arc<vulkano::descriptor::pipeline_layout::PipelineLayoutAbstract + Sync + Send>>,
    dimensions: [u32; 2],
    sampler: Arc<vulkano::sampler::Sampler>,
}
```

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.
