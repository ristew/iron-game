use std::{collections::{HashMap, HashSet}, f32::consts::PI};

use lazy_static::__Deref;
use noise::{Fbm, HybridMulti, NoiseFn, Perlin};
use rand::{Rng, random, thread_rng};
use rand_distr::Uniform;

use crate::{self::*, settlement::Settlement};

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
    let mut ocean_map: HashSet<Coordinate> = HashSet::new();
    for (&coordinate, &height) in height_map.iter() {
        let terrain = if height > 0.0 {
            Terrain::Hills
        } else {
            Terrain::Ocean
        };
        let province_id = world.insert_province(Province {
            id: None,
            terrain,
            climate: Climate::Mild,
            coordinate,
            harvest_month: 8,
            settlements: Vec::new(),
            features: HashSet::new(),
            controller: None,
            coastal: false,
        });
        if terrain == Terrain::Ocean {
            ocean_map.insert(coordinate);
        }
    }
    for &coordinate in height_map.keys() {
        let province_id = world.get_province_coordinate(coordinate).unwrap();
        let is_ocean = province_id.get().terrain == Terrain::Ocean;

        for other_coord in coordinate.neighbors_iter() {
            if let Some(other_province) = world.get_province_coordinate(other_coord) {
                if is_ocean ^ (other_province.get().terrain == Terrain::Ocean) {
                    province_id.get_mut().coastal = true;
                    other_province.get_mut().coastal = true;
                }
            }
        }
    }
}

pub fn create_test_world(world: &mut World) {
    generate_world(world);
    let religion_id = world.insert(Religion {
        id: None,
        name: "Test Religion".to_owned(),
    });

    let mut language = Language::new();
    language.name = language.generate_name(2);
    let culture_name = language.generate_name(2);
    let language_id = world.insert(language);
    let culture_id = world.insert(Culture {
        id: None,
        name: culture_name,
        language: language_id.clone(),
        religion: religion_id.clone(),
        features: Vec::new(),
    });

    // create provinces
    for i in 0..100 {
        for j in 0..100 {
            let coordinate = Coordinate::new(i - (j / 2), j);

            let province_id = world.get_province_coordinate(coordinate).unwrap();

            if province_id.get().terrain == Terrain::Ocean {
                continue;
            }

            if random::<f32>() > 0.9 {
                let polity_id = add_polity(world, language_id.get().generate_name(2), culture_id.clone(), PolityLevel::Tribe);
                add_test_settlement(world, culture_id.clone(), province_id.clone(), polity_id);
            }
        }
    }
}

fn add_test_settlement(world: &mut World, culture: Culture, province: Province, polity: Polity) -> Settlement {
    let name = culture.langauage(world).generate_name(3);
    world.insert_settlement(name, culture, province, polity, 100)
}
