use alga::general::SubsetOf;

pub fn create_static_draw_w(pos: ::na::Isometry3<f32>, radius: f32, primitive: ::graphics::Primitive, world: &mut ::specs::World) {
    let world_trans = {
        let trans: ::na::Transform3<f32> =
            ::na::Similarity3::from_isometry(pos, radius).to_superset();
        ::graphics::shader::draw1_vs::ty::World {
            world: trans.unwrap().into(),
        }
    };

    let entity = world.create_entity().build();
    let (primitive, groups) = primitive.instantiate();

    ::component::StaticDraw::add(
        entity,
        primitive,
        groups,
        ::graphics::Color::Red,
        world_trans,
        &mut world.write(),
        &world.read_resource(),
    );
}

pub fn draw_score(pos: ::na::Isometry3<f32>, world: &mut ::specs::World) {
    let radius = 0.05;

    let mut p = vec![
        (::graphics::Primitive::TextBestScores, 0, 0),
        (::graphics::Primitive::TextLastScores, 40, 0),
    ];

    for i in 1..12 {
        p.push((::graphics::Primitive::TextUnderScore, 0*3+4, -i*6));
        p.push((::graphics::Primitive::TextUnderScore, 1*3+4, -i*6));
        p.push((     ::graphics::Primitive::TextColon, 2*3+4, -i*6));
        p.push((::graphics::Primitive::TextUnderScore, 3*3+4, -i*6));
        p.push((::graphics::Primitive::TextUnderScore, 4*3+4, -i*6));
        p.push((     ::graphics::Primitive::TextColon, 5*3+4, -i*6));
        p.push((::graphics::Primitive::TextUnderScore, 6*3+4, -i*6));
        p.push((::graphics::Primitive::TextUnderScore, 7*3+4, -i*6));

        p.push((::graphics::Primitive::TextUnderScore, 0*3+44, -i*6));
        p.push((::graphics::Primitive::TextUnderScore, 1*3+44, -i*6));
        p.push((     ::graphics::Primitive::TextColon, 2*3+44, -i*6));
        p.push((::graphics::Primitive::TextUnderScore, 3*3+44, -i*6));
        p.push((::graphics::Primitive::TextUnderScore, 4*3+44, -i*6));
        p.push((     ::graphics::Primitive::TextColon, 5*3+44, -i*6));
        p.push((::graphics::Primitive::TextUnderScore, 6*3+44, -i*6));
        p.push((::graphics::Primitive::TextUnderScore, 7*3+44, -i*6));
    }

    let trans: ::na::Transform3<f32> = ::na::Similarity3::from_isometry(pos, radius).to_superset();
    for (primitive, dx, dy) in p {
        let local_trans = ::na::Translation3::new(dx as f32 * ::graphics::font::POINT_CENTER_DISTANCE, dy as f32 * ::graphics::font::POINT_CENTER_DISTANCE, 0.0);
        let world_trans = {
            ::graphics::shader::draw1_vs::ty::World {
                world: (trans*local_trans).unwrap().into(),
            }
        };

        let entity = world.create_entity().build();
        let (primitive, groups) = primitive.instantiate();

        ::component::StaticDraw::add(
            entity,
            primitive,
            groups,
            ::graphics::Color::Red,
            world_trans,
            &mut world.write(),
            &world.read_resource(),
        );
    }
}