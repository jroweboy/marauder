// See LICENSE file for copyright and license details.

use cgmath::vector::Vector;
use geom::Geom;
use core_types::{
    Bool,
    Int,
    MapPos,
    UnitId,
};
use core::{
    Unit,
};
use gl_types::{
    Scene,
    SceneNode,
    Float,
    WorldPos,
};
use game_state::GameState;

pub trait EventVisualizer {
    fn is_finished(&self) -> Bool;
    fn draw(&mut self, geom: &Geom, scene: &mut Scene);
    fn end(&mut self, geom: &Geom, scene: &mut Scene, game_state: &mut GameState);
}

static MOVE_SPEED: Float = 40.0; // TODO: config?

pub struct EventMoveVisualizer {
    unit_id: UnitId,
    path: ~[MapPos],
    current_move_index: Int,
}

impl EventVisualizer for EventMoveVisualizer {
    fn is_finished(&self) -> Bool {
        assert!(self.current_move_index <= self.frames_count());
        self.current_move_index == self.frames_count()
    }

    fn draw(&mut self, geom: &Geom, scene: &mut Scene) {
        let node = scene.get_mut(&self.unit_id);
        node.pos = self.current_position(geom);
        self.current_move_index += 1;
    }

    fn end(&mut self, geom: &Geom, scene: &mut Scene, game_state: &mut GameState) {
        let unit_node = scene.get_mut(&self.unit_id);
        unit_node.pos = self.current_position(geom);
        let unit = game_state.units.mut_iter().find(|u| u.id == self.unit_id).unwrap();
        unit.pos = *self.path.last().unwrap();
    }
}

impl EventMoveVisualizer {
    pub fn new(unit_id: UnitId, path: ~[MapPos]) -> ~EventVisualizer {
        ~EventMoveVisualizer {
            unit_id: unit_id,
            path: path,
            current_move_index: 0,
        } as ~EventVisualizer
    }

    fn frames_count(&self) -> Int {
        let len = self.path.len() as Int - 1;
        len * MOVE_SPEED as Int
    }

    fn current_tile(&self) -> MapPos {
        self.path[self.current_tile_index()]
    }

    fn next_tile(&self) -> MapPos {
        self.path[self.current_tile_index() + 1]
    }

    fn current_tile_index(&self) -> Int {
        // self.current_move_index / MOVE_SPEED as Int
        0
    }

    fn node_index(&self) -> Int {
        // self.current_move_index - self.current_tile_index() * MOVE_SPEED
        self.current_move_index
    }

    fn current_position(&self, geom: &Geom) -> WorldPos {
        let from = geom.map_pos_to_world_pos(self.current_tile());
        let to = geom.map_pos_to_world_pos(self.next_tile());
        let diff = to.sub_v(&from).div_s(MOVE_SPEED);
        from.add_v(&diff.mul_s(self.node_index() as Float))
    }
}

pub struct EventEndTurnVisualizer;

impl EventEndTurnVisualizer {
    pub fn new() -> ~EventVisualizer {
        ~EventEndTurnVisualizer as ~EventVisualizer
    }
}

impl EventVisualizer for EventEndTurnVisualizer {
    fn is_finished(&self) -> Bool {
        true
    }

    fn draw(&mut self, _: &Geom, _: &mut Scene) {}

    fn end(&mut self, _: &Geom, _: &mut Scene, _: &mut GameState) {}
}

pub struct EventCreateUnitVisualizer {
    id: UnitId,
    pos: MapPos,
}

impl EventCreateUnitVisualizer {
    pub fn new(id: UnitId, pos: MapPos) -> ~EventVisualizer {
        ~EventCreateUnitVisualizer {
            id: id,
            pos: pos,
        } as ~EventVisualizer
    }
}

impl EventVisualizer for EventCreateUnitVisualizer {
    fn is_finished(&self) -> Bool {
        true
    }

    fn draw(&mut self, _: &Geom, _: &mut Scene) {}

    fn end(&mut self, geom: &Geom, scene: &mut Scene, game_state: &mut GameState) {
        let world_pos = geom.map_pos_to_world_pos(self.pos);
        scene.insert(self.id, SceneNode{pos: world_pos});
        assert!(game_state.units.iter().find(|u| u.id == self.id).is_none());
        game_state.units.push(Unit{id: self.id, pos: self.pos});
    }
}
// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
