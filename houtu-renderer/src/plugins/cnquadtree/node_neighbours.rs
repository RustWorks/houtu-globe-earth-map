use super::direction::Direction;
use super::terrain_quadtree_node::TerrainQuadtreeNode;
use std::fmt;
use std::ops::Index;
use std::ops::IndexMut;
#[derive(Clone, Copy, PartialEq)]
pub struct NodeNeighbours {
    pub(super) north: TerrainQuadtreeNode,
    pub(super) east: TerrainQuadtreeNode,
    pub(super) south: TerrainQuadtreeNode,
    pub(super) west: TerrainQuadtreeNode,
}

impl Default for NodeNeighbours {
    fn default() -> Self {
        Self {
            north: TerrainQuadtreeNode::None,
            east: TerrainQuadtreeNode::None,
            south: TerrainQuadtreeNode::None,
            west: TerrainQuadtreeNode::None,
        }
    }
}

impl Index<Direction> for NodeNeighbours {
    type Output = TerrainQuadtreeNode;

    fn index(&self, dir: Direction) -> &TerrainQuadtreeNode {
        match dir {
            Direction::North => &self.north,
            Direction::East => &self.east,
            Direction::South => &self.south,
            Direction::West => &self.west,
        }
    }
}

impl IndexMut<Direction> for NodeNeighbours {
    fn index_mut(&mut self, dir: Direction) -> &mut TerrainQuadtreeNode {
        match dir {
            Direction::North => &mut self.north,
            Direction::East => &mut self.east,
            Direction::South => &mut self.south,
            Direction::West => &mut self.west,
        }
    }
}
