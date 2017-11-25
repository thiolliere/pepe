pub fn create_teleport<'a>(
    pos: ::na::Isometry3<f32>,
    teleports: &mut ::specs::WriteStorage<'a, ::component::Teleport>,
    proximitors: &mut ::specs::WriteStorage<'a, ::component::Proximitor>,
    sensors: &mut ::specs::WriteStorage<'a, ::component::PhysicSensor>,
    physic_world: &mut ::specs::FetchMut<'a, ::resource::PhysicWorld>,
    entities: &::specs::Entities,
) {
    let shape = ::ncollide::shape::Cuboid::new(::na::Vector3::new(0.4, 0.4, 0.0));
    let pos = pos * ::na::Translation3::from_vector(::na::Vector3::new(0.0, 0.0, -0.4));

    let mut group = ::nphysics::object::SensorCollisionGroups::new();
    group.set_whitelist(&[super::PLAYER_GROUP]);

    let mut sensor = ::nphysics::object::Sensor::new(shape, None);
    sensor.set_relative_position(pos);
    sensor.set_collision_groups(group);

    let entity = entities.create();
    proximitors.insert(entity, ::component::Proximitor::new());
    teleports.insert(entity, ::component::Teleport);
    ::component::PhysicSensor::add(entity, sensor, sensors, physic_world);
}