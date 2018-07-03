use specs;
use ncollide2d;
use specs::Join;

pub struct PhysicSystem;

impl<'a> specs::System<'a> for PhysicSystem {
    type SystemData = (
        specs::WriteStorage<'a, ::component::Contactor>,
        specs::ReadExpect<'a, ::resource::UpdateTime>,
        specs::ReadExpect<'a, ::resource::BodiesMap>,
        specs::WriteExpect<'a, ::resource::PhysicWorld>,
    );

    fn run(
        &mut self,
        (
            mut contactors,
            update_time,
            bodies_map,
            mut physic_world,
        ): Self::SystemData,
    ) {
        physic_world.set_timestep(update_time.0);
        physic_world.step();
        for contact in physic_world.contact_events() {
            let collision_world = physic_world.collision_world();
            match contact {
                &ncollide2d::events::ContactEvent::Started(coh1, coh2) => {
                    let bh1 = collision_world
                        .collision_object(coh1)
                        .unwrap()
                        .data()
                        .body();
                    let bh2 = collision_world
                        .collision_object(coh2)
                        .unwrap()
                        .data()
                        .body();
                    let e1 = *bodies_map.get(&bh1).unwrap();
                    let e2 = *bodies_map.get(&bh2).unwrap();
                    if let Some(contactor) = contactors.get_mut(e1) {
                        contactor.push(e2);
                    }
                    if let Some(contactor) = contactors.get_mut(e2) {
                        contactor.push(e1);
                    }
                }
                _ => (),
            }
        }
    }
}

pub struct AirjumpSystem;

impl<'a> specs::System<'a> for AirjumpSystem {
    type SystemData = (
        specs::ReadStorage<'a, ::component::AirjumpRestorer>,
        specs::ReadStorage<'a, ::component::Contactor>,
        specs::WriteStorage<'a, ::component::Airjump>,
    );

    fn run(
        &mut self,
        (
            restorers,
            contactors,
            mut airjumps,
        ): Self::SystemData,
    ) {
        for (airjump, contactor) in (&mut airjumps, &contactors).join() {
            for contact in contactor.iter() {
                if restorers.get(*contact).is_some() {
                    airjump.0 = true;
                }
            }
        }
    }
}
