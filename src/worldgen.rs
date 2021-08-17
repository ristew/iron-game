use std::{collections::HashMap, f32::consts::PI};

use noise::{Fbm, HybridMulti, NoiseFn, Perlin};
use rand::{thread_rng, Rng};
use rand_distr::Uniform;

use crate::*;

const MAP_SIZE: isize = 100;

fn center_dist(coordinate: Coordinate) -> f32 {
    coordinate.dist(Coordinate::new(MAP_SIZE / 4, MAP_SIZE / 2)) as f32
}

fn generate_height_map() -> HashMap<Coordinate, f32> {
    /*
     * add perlin noise to basin
     */
    let mut height_map: HashMap<Coordinate, f32> = HashMap::new();
    let perlin = Perlin::new();
    let fbm = Fbm::new();
    for i in 0..MAP_SIZE {
        for j in 0..MAP_SIZE {
            let coordinate = Coordinate::new(i - (j / 2), j);
            let bpp = coordinate.base_pixel_pos();
            let basin_height = (3.0 * PI * center_dist(coordinate) / MAP_SIZE as f32)
                .sin()
                .powf(3.0)
                - 0.1;
            let noise = fbm.get([
                bpp.x as f64 / (5.0 * TILE_SIZE_X as f64),
                bpp.y as f64 / (5.0 * TILE_SIZE_Y as f64),
            ]) as f32;
            height_map.insert(coordinate, noise + basin_height);
        }
    }
    height_map
}

pub fn generate_world(world: &mut World) {
    let height_map = generate_height_map();
    for (&coordinate, &height) in height_map.iter() {
        let province_id = world.storages.get_id::<Province>();
        let terrain = if height > 0.0 {
            Terrain::Hills
        } else {
            Terrain::Ocean
        };
        world.insert_province(Province {
            id: province_id.clone(),
            terrain,
            climate: Climate::Mild,
            coordinate,
            harvest_month: 8,
            settlements: Vec::new(),
        });
    }
}

pub fn create_test_world(world: &mut World) {
    generate_world(world);
    let culture_id = world.storages.get_id::<Culture>();
    let religion_id = world.storages.get_id::<Religion>();
    let language_id = world.storages.get_id::<Language>();

    world.insert(Religion {
        id: religion_id.clone(),
        name: "Test Religion".to_owned(),
    });

    let mut language = Language::new(language_id.clone());
    world.insert(Culture {
        id: culture_id.clone(),
        language: language_id.clone(),
        name: language.generate_name(2),
        religion: religion_id.clone(),
        features: Vec::new(),
    });
    language.name = language.generate_name(2);
    world.insert(language);

    // create provinces
    for i in 0..100 {
        for j in 0..100 {
            let coordinate = Coordinate::new(i - (j / 2), j);

            let province_id = world.get_province_coordinate(coordinate).unwrap();

            if province_id.get(world).borrow().terrain == Terrain::Ocean {
                continue;
            }

            for i in 0..thread_rng().sample(Uniform::new(0, 2)) {
                add_test_settlement(world, culture_id.clone(), province_id.clone());
            }
        }
    }
}

fn add_test_settlement(world: &mut World, culture_id: CultureId, province_id: ProvinceId) {
    let settlement_id = world.storages.get_id::<Settlement>();
    let pop_id = world.storages.get_id::<Pop>();

    let pop = world.insert(Pop {
        id: pop_id.clone(),
        size: 100,
        farmed_good: Some(Wheat),
        culture: culture_id.clone(),
        settlement: settlement_id.clone(),
        province: province_id.clone(),
        satiety: Satiety {
            base: 0.0,
            luxury: 0.0,
        },
        kid_buffer: KidBuffer::new(),
        owned_goods: GoodStorage(HashMap::new()),
        migration_status: None,
    });

    world
        .get_ref::<Pop>(&pop)
        .borrow_mut()
        .owned_goods
        .add(Wheat, 30000.0);

    let name = culture_id
        .get(world)
        .borrow()
        .language
        .get(world)
        .borrow()
        .generate_name(4);
    world.insert_settlement(Settlement {
        id: settlement_id.clone(),
        name,
        pops: vec![pop_id.clone()],
        features: Vec::new(),
        primary_culture: culture_id.clone(),
        province: province_id.clone(),
        level: SettlementLevel::Village,
    });
}
