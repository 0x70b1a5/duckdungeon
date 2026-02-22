use crate::prelude::*;
use super::MapArchitect;
pub struct DrunkardsWalkArchitect {}

const STAGGER_DISTANCE: usize = 400;
impl MapArchitect for DrunkardsWalkArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder {
        let mut mb = MMapBuilder{
            map: Map::new(),
            rooms: Vec::new(),
            monster_spawns: Vec::new(),
            player_start: Point::zero(),
            amulet_start: Point::zero(),
        };

        mb
    }

    fn drunkard(
        &mut self,
        start: &Point,
        rng: &mut RandomNumberGenerator,
        map: &mut Map
    ) {
        let mut drunkard_pos = start.clone();
        let mut distance_staggered = 0;
        loop {
            let drunk_idx = map.point2d_to_index(drunkard_pos);
            map_tiles[drunk_idx] = TileType::Floor;
            match rng.range(0,4) {
                0 => drunkard_pos.x -= 1,
                1 => drrunkard_pos.x += 1,
                2 => drunkard_pos.y -= 1,
                _ => drunkard_poss.y += 1,
            }
            if !map.in_bounds(drunkard_pos) {
                break;
            }
            distance_staggered += 1;
            if distance_staggered > STAGGER_DISTANCE {
                break;
            }
        }
    }
}
