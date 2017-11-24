pub fn create_player<'a>(
    pos: ::na::Vector3<f32>,
    players: &mut ::specs::WriteStorage<'a, ::component::Player>,
    aims: &mut ::specs::WriteStorage<'a, ::component::Aim>,
    momentums: &mut ::specs::WriteStorage<'a, ::component::Momentum>,
    bodies: &mut ::specs::WriteStorage<'a, ::component::PhysicBody>,
    shooters: &mut ::specs::WriteStorage<'a, ::component::Shooter>,
    weapon_animations: &mut ::specs::WriteStorage<'a, ::component::WeaponAnimation>,
    weapon_anchors: &mut ::specs::WriteStorage<'a, ::component::WeaponAnchor>,
    dynamic_huds: &mut ::specs::WriteStorage<'a, ::component::DynamicHud>,
    dynamic_graphics_assets: &mut ::specs::WriteStorage<'a, ::component::DynamicGraphicsAssets>,
    physic_world: &mut ::specs::FetchMut<'a, ::resource::PhysicWorld>,
    entities: &::specs::Entities,
) {
    let shape = ::ncollide::shape::Cylinder::new(0.4, 0.1);
    let pos = ::na::Isometry3::new(pos, ::na::Vector3::x() * ::std::f32::consts::FRAC_PI_2);

    let mut group = ::nphysics::object::RigidBodyCollisionGroups::new_dynamic();
    group.set_membership(&[super::ALIVE_GROUP, super::PLAYER_GROUP]);

    let mut body = ::nphysics::object::RigidBody::new_dynamic(shape, 1.0, 0.0, 0.0);
    body.set_transformation(pos);
    body.set_collision_groups(group);

    let mass = 1.0 / body.inv_mass();
    let velocity = 10.0;
    let time_to_reach_v_max = 0.1;
    let ang_damping = 0.0;

    let entity = entities.create();
    players.insert(entity, ::component::Player);
    aims.insert(entity, ::component::Aim::new());
    momentums.insert(
        entity,
        ::component::Momentum::new(mass, velocity, time_to_reach_v_max, ang_damping, None),
    );
    super::create_weapon(
        entity,
        shooters,
        weapon_animations,
        weapon_anchors,
        dynamic_huds,
        dynamic_graphics_assets,
        entities,
    );

    ::component::PhysicBody::add(entity, body, bodies, physic_world);
}
