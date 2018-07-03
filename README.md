# Airjump Multi

This is a basic game showing a way to structure your code with `specs`, `nphysics` (and `vulkano`).

Each users control a ball that have one airjump, this airjump is restored on contact with the ground and other balls. The goal is to touch the gong.

This README will explain some concept about the implementation. The most interesting part is integrate nphysics with specs.
TODO: CI

## Entity/Component/System (ECS)

ECS is a common pattern used in games.
Entities are sets of components, Components are structure that holds datas and Systems are function that iterate on components to update their data.

In this game there is:
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
    * `Controller`: uses gamepad event and sets airjumps impulse

* Entities:
  * ball: `Image`, `Control`, `Airjump`, `AirjumpRestorer`, `Contactor`, `RigidBody`
  * gong: `Image`, `RigidBody`
  * walls: `RigidBody`
  * ground: `AirjumpRestorer`, `RigidBody`

## nphysics and specs

This is probably the most interesting part.

//TODO

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
