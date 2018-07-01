use gilrs;
use specs;

pub trait GameState {
    fn update(self: Box<Self>, world: &mut specs::World) -> Box<GameState>;
    fn event(self: Box<Self>, event: gilrs::EventType, world: &mut specs::World)
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

pub struct Start;

impl GameState for Start {
    fn update(self: Box<Self>, world: &mut specs::World) -> Box<GameState> {
        world.write_resource::<::resource::DrawImage>().0 = Some(::Image::Start);
        self
    }
    fn event(self: Box<Self>, _event: gilrs::EventType, _world: &mut specs::World)
        -> Box<GameState>
    {
        self
    }
    fn gamepad(
        self: Box<Self>,
        id: usize,
        _gamepad: &::gilrs::Gamepad,
        _world: &mut specs::World,
    ) -> Box<GameState>
    {
        Box::new(NewController { id })
    }
    fn paused(&self) -> bool {
        true
    }
}

pub struct NewController {
    id: usize,
}

impl GameState for NewController {
    fn update(self: Box<Self>, world: &mut specs::World) -> Box<GameState> {
        world.write_resource::<::resource::DrawImage>().0 = Some(::Image::NewController);
        self
    }
    fn event(self: Box<Self>, _event: gilrs::EventType, _world: &mut specs::World)
        -> Box<GameState>
    {
        self
    }
    fn gamepad(
        self: Box<Self>,
        id: usize,
        gamepad: &::gilrs::Gamepad,
        _world: &mut specs::World,
    ) -> Box<GameState>
    {
        if id == self.id {
            if gamepad.is_pressed(gilrs::ev::Button::West) {
                Box::new(ShowImage::new(::Image::NewController1))
            } else if gamepad.is_pressed(gilrs::ev::Button::South) {
                Box::new(ShowImage::new(::Image::NewController2))
            } else if gamepad.is_pressed(gilrs::ev::Button::East) {
                Box::new(ShowImage::new(::Image::NewControllerSkip))
            } else {
                self
            }
        } else {
            self
        }
    }
    fn paused(&self) -> bool {
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
    fn event(self: Box<Self>, _event: gilrs::EventType, _world: &mut specs::World)
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
    fn paused(&self) -> bool {
        true
    }
}

pub struct Play;

impl GameState for Play {
    fn update(self: Box<Self>, _world: &mut specs::World) -> Box<GameState> {
        self
    }
    fn event(self: Box<Self>, _event: gilrs::EventType, _world: &mut specs::World)
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
    fn paused(&self) -> bool {
        false
    }
}
