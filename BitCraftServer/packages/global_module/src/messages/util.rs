use spacetimedb::SpacetimeType;

use crate::SurfaceType;

#[derive(SpacetimeType, Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct FloatHexTileMessage {
    pub x: i32,
    pub z: i32,
    pub dimension: u32,
}

#[derive(SpacetimeType, Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct SmallHexTileMessage {
    pub x: i32,
    pub z: i32,
    pub dimension: u32,
}

#[derive(SpacetimeType, Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct LargeHexTileMessage {
    pub x: i32,
    pub z: i32,
    pub dimension: u32,
}

#[derive(SpacetimeType, Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct OffsetCoordinatesFloat {
    pub x: i32,
    pub z: i32,
    pub dimension: u32,
}

#[derive(SpacetimeType, Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct OffsetCoordinatesSmallMessage {
    pub x: i32,
    pub z: i32,
    pub dimension: u32,
}

#[derive(SpacetimeType, Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct OffsetCoordinatesLargeMessage {
    pub x: i32,
    pub z: i32,
    pub dimension: u32,
}

#[derive(SpacetimeType, PartialEq, Eq, Clone, Copy, Debug)]
pub struct ChunkCoordinatesMessage {
    pub x: i32,
    pub z: i32,
    pub dimension: u32,
}

#[derive(SpacetimeType, Default, Copy, Clone, Debug, PartialEq)]
pub struct MovementSpeed {
    pub surface_type: SurfaceType,
    pub speed: f32,
}
