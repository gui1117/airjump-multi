use gilrs;
use specs;
use specs::Join;
use nphysics2d;
use entity;

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
    fn paused(&self, world: &specs::World) -> bool;
}

pub struct NewController {
    id: usize,
}

impl GameState for NewController {
    fn update(self: Box<Self>, world: &mut specs::World) -> Box<GameState> {
        world.write_resource::<::resource::DrawImage>().0 = Some(::Image::NewController);
        self
    }
    fn event(self: Box<Self>, event: gilrs::ev::Event, world: &mut specs::World)
        -> Box<GameState>
    {
        if event.id == self.id {
            match event.event {
                gilrs::ev::EventType::Disconnected => {
                    Box::new(Play)
                },
                gilrs::ev::EventType::ButtonPressed(gilrs::ev::Button::West, _) => {
                    entity::create_ball(event.id, [true, true], world);
                    Box::new(ShowImage::new(::Image::NewController1))
                },
                gilrs::ev::EventType::ButtonPressed(gilrs::ev::Button::South, _) => {
                    entity::create_ball(event.id, [true, false], world);
                    entity::create_ball(event.id, [false, true], world);
                    Box::new(ShowImage::new(::Image::NewController2))
                },
                gilrs::ev::EventType::ButtonPressed(gilrs::ev::Button::East, _) => {
                    Box::new(ShowImage::new(::Image::NewControllerSkip))
                },
                _ => self
            }
        } else {
            self
        }
    }
    fn gamepad(
        self: Box<Self>,
        _id: usize,
        _gamepad: &::gilrs::Gamepad,
        _world: &mut specs::World,
    ) -> Box<GameState>
    {
        self
    }
    fn paused(&self, _world: &specs::World) -> bool {
        true
    }
}

pub struct ShowImage {
    time: usize,
    image: ::Image,
}

impl ShowImage {
    fn new(image: ::Image) -> Self {
        ShowImage {
            image,
            time: 30,
        }
    }
}

impl GameState for ShowImage {
    fn update(mut self: Box<Self>, world: &mut specs::World) -> Box<GameState> {
        world.write_resource::<::resource::DrawImage>().0 = Some(self.image);
        self.time -= 1;
        if self.time == 0 {
            Box::new(Play)
        } else {
            self
        }
    }
    fn event(self: Box<Self>, _event: gilrs::ev::Event, _world: &mut specs::World)
        -> Box<GameState>
    {
        self
    }
    fn gamepad(
        self: Box<Self>,
        _id: usize,
        _gamepad: &::gilrs::Gamepad,
        _world: &mut specs::World,
    ) -> Box<GameState>
    {
        self
    }
    fn paused(&self, _world: &specs::World) -> bool {
        true
    }
}

pub struct Play;

impl GameState for Play {
    fn update(self: Box<Self>, world: &mut specs::World) -> Box<GameState> {
        let controls = world.read_storage::<::component::Control>();
        let count = controls.join().count();
        if count == 0 {
            world.write_resource::<::resource::DrawImage>().0 = Some(::Image::Start);
        } else if count == 1 {
            world.write_resource::<::resource::DrawImage>().0 = Some(::Image::Wait);
        }
        self
    }
    fn event(self: Box<Self>, event: gilrs::ev::Event, world: &mut specs::World)
        -> Box<GameState>
    {
        let controls = world.read_storage::<::component::Control>();
        if controls.join().filter(|c| c.gamepad_id == event.id).next().is_none() {
            return Box::new(NewController { id: event.id })
        }
        self
    }
    fn gamepad(
        self: Box<Self>,
        id: usize,
        gamepad: &::gilrs::Gamepad,
        world: &mut specs::World,
    ) -> Box<GameState>
    {
        let controls = world.read_storage::<::component::Control>();
        let bodies = world.read_storage::<::component::RigidBody>();
        let mut airjumps = world.write_storage::<::component::Airjump>();
        let mut physic_world = world.write_resource::<::resource::PhysicWorld>();

        for (c, airjump, body) in (&controls, &mut airjumps, &bodies).join().filter(|(c, _, _)| c.gamepad_id == id) {
            let body = body.get_mut(&mut physic_world);
            // Set angle
            let mut v = ::na::Vector2::new(0.0, 0.0);
            if c.parts[0] {
                v[0] += gamepad.value(gilrs::ev::Axis::LeftStickX);
                v[1] += gamepad.value(gilrs::ev::Axis::LeftStickY);
            }
            if c.parts[1] {
                v[0] += gamepad.value(gilrs::ev::Axis::RightStickX);
                v[1] += gamepad.value(gilrs::ev::Axis::RightStickY);
            }
            if let Some(v) = v.try_normalize(0.0001) {
                let current_angle = body.position().rotation.angle();
                let next_angle = -v[1].atan2(v[0]);
                body.apply_displacement(&nphysics2d::math::Velocity::angular(next_angle - current_angle));
            }

            // Jump
            // IDEA: add to velocity instead of reset it
            if (gamepad.is_pressed(gilrs::ev::Button::LeftTrigger) && c.parts[0])
                || (gamepad.is_pressed(gilrs::ev::Button::RightTrigger) && c.parts[1])
            {
                if airjump.0 {
                    airjump.0 = false;
                    let angle = body.position().rotation.angle();
                    body.set_velocity(nphysics2d::math::Velocity::linear(angle.cos()*entity::BALL_VELOCITY, angle.sin()*entity::BALL_VELOCITY));
                }
            }
        }
        self
    }
    fn paused(&self, world: &specs::World) -> bool {
        let controls = world.read_storage::<::component::Control>();
        let count = controls.join().count();
        count < 2
    }
}
