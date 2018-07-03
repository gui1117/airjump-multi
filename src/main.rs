extern crate image;
extern crate hibitset;
extern crate gilrs;
extern crate winit;
extern crate specs;
extern crate nphysics2d;
extern crate ncollide2d;
extern crate alga;
extern crate fnv;
#[macro_use]
extern crate derive_deref;
#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;
#[macro_use]
extern crate enum_iterator_derive;
#[macro_use]
extern crate specs_derive;
extern crate nalgebra as na;

mod graphics;
mod resource;
mod component;
mod entity;
mod system;
mod state;
mod retained_storage;

#[derive(EnumIterator, Eq, PartialEq, Hash, Clone, Copy)]
pub enum Image {
    Start,
    Wait,
    NewController,
    NewControllerSkip,
    NewController1,
    NewController2,
    Wallpaper,
    Ball,
    Gong,
}
impl Image {
    pub fn data(&self) -> &[u8] {
        match self {
            Image::Start => include_bytes!("../assets/Start.png"),
            Image::Wait => include_bytes!("../assets/Wait.png"),
            Image::NewController => include_bytes!("../assets/NewController.png"),
            Image::NewControllerSkip => include_bytes!("../assets/NewControllerSkip.png"),
            Image::NewController1 => include_bytes!("../assets/NewController1.png"),
            Image::NewController2 => include_bytes!("../assets/NewController2.png"),
            Image::Wallpaper => include_bytes!("../assets/Wallpaper.png"),
            Image::Ball => include_bytes!("../assets/Ball.png"),
            Image::Gong => include_bytes!("../assets/Gong.png"),
        }
    }
}

pub fn safe_maintain(world: &mut specs::World) {
    use retained_storage::Retained;

    world.maintain();
    let mut physic_world = world.write_resource::<::resource::PhysicWorld>();
    let mut bodies_map = world.write_resource::<::resource::BodiesMap>();

    let retained = world
        .write_storage::<::component::RigidBody>()
        .retained()
        .iter()
        .map(|r| r.0)
        .collect::<Vec<_>>();
    physic_world.remove_bodies(&retained);
    for handle in &retained {
        bodies_map.remove(handle);
    }
}

fn main() {
    ::std::env::set_var("WINIT_UNIX_BACKEND", "x11");
    let mut events_loop = winit::EventsLoop::new();
    let mut graphics = graphics::Graphics::new(&events_loop);
    let mut gilrs = gilrs::Gilrs::new().unwrap();

    let mut physic_world = ::resource::PhysicWorld::new();
    physic_world.set_gravity(nphysics2d::math::Vector::new(0.0, entity::GRAVITY));

    let mut world = specs::World::new();
    world.register::<::component::RigidBody>();
    world.register::<::component::Contactor>();
    world.register::<::component::Airjump>();
    world.register::<::component::AirjumpRestorer>();
    world.register::<::component::Control>();
    world.register::<::component::Image>();
    world.add_resource(::resource::UpdateTime(0.0));
    world.add_resource(::resource::DrawImage(None));
    world.add_resource(::resource::BodiesMap::new());
    world.add_resource(physic_world);
    let mut update_dispatcher = specs::DispatcherBuilder::new()
        .with(::system::PhysicSystem, "physic", &[])
        .with(::system::AirjumpSystem, "airjump", &["physic"])
        .build();

    entity::create_gong(&mut world);
    entity::create_ground(&mut world);
    entity::create_walls(&mut world);

    let mut last_frame_instant = std::time::Instant::now();
    let mut last_update_instant = std::time::Instant::now();

    let mut state = Box::new(state::Start) as Box<state::GameState>;
    loop {
        // Poll events
        let mut done = false;
        events_loop.poll_events(|ev| match ev {
            winit::Event::WindowEvent {
                event: winit::WindowEvent::Closed,
                ..
            } |
            winit::Event::WindowEvent {
                event: winit::WindowEvent::KeyboardInput {
                    input: winit::KeyboardInput {
                        virtual_keycode: Some(winit::VirtualKeyCode::Escape),
                        ..
                    },
                    ..
                },
                ..
            } => done = true,
            _ => (),
        });
        if done {
            return;
        }
        while let Some(ev) = gilrs.next_event() {
            gilrs.update(&ev);
            state = state.event(ev, &mut world);
        }
        for (id, gamepad) in gilrs.gamepads() {
            state = state.gamepad(id, gamepad, &mut world);
        }
        if state.quit() {
            return;
        }

        // Update world
        let delta_time = last_update_instant.elapsed();
        last_update_instant = std::time::Instant::now();
        if !state.paused() {
            world.write_resource::<::resource::UpdateTime>().0 = delta_time
                .as_secs()
                .saturating_mul(1_000_000_000)
                .saturating_add(delta_time.subsec_nanos() as u64)
                as f32 / 1_000_000_000.0;
            update_dispatcher.dispatch(&mut world.res);
        } else {
            world.write_resource::<::resource::UpdateTime>().0 = 0.0;
            // pause_dispatcher.dispatch(&mut world.res);
        }
        state = state.update(&mut world);
        safe_maintain(&mut world);

        // Draw world
        graphics.render(&mut world);

        // Sleep
        let elapsed = last_frame_instant.elapsed();
        let frame_duration = {
            std::time::Duration::new(0, (1_000_000_000.0 / 60 as f32) as u32)
        };
        if let Some(to_sleep) = frame_duration.checked_sub(elapsed) {
            std::thread::sleep(to_sleep);
        }
        last_frame_instant = std::time::Instant::now();
    }
}
