use spacetimedb::SpacetimeType;

use super::components::{BuildingState, DroppedInventoryState, EnemyState, InventoryState, NpcState, ResourceState};

#[derive(SpacetimeType)]
pub struct WorldGenWorldDefinition {
    pub size: WorldGenVector2Int,
    pub land_curve: WorldGenAnimationCurve,
    pub noise_influence: f32,
    pub sea_level: i32,
    pub world_map: WorldGenWorldMapDefinition,
    pub biomes_map: WorldGenBiomesMapDefinition,
    pub mountains_map: WorldGenMountainsMapDefinition,
    pub buildings_map: WorldGenBuildingsMapDefinition,
    pub resources_map: WorldGenResourcesMapDefinition,
}

#[derive(SpacetimeType, Default, Debug)]
pub struct WorldGenAnimationCurve {
    pub keyframes: Vec<WorldGenAnimationCurveKeyframe>,
}

#[derive(SpacetimeType, Clone, Default, Debug)]
pub struct WorldGenVector2Int {
    pub x: i32,
    pub y: i32,
}

#[derive(SpacetimeType)]
pub struct WorldGenWorldMapDefinition {
    pub debug_step: i32,
    pub shapes: Vec<WorldGenLandShapeDefinition>,
}

#[derive(SpacetimeType)]
pub struct WorldGenBiomesMapDefinition {
    pub biomes: Vec<WorldGenBiomeDefinition>,
    pub values: Vec<u8>,
}

#[derive(SpacetimeType)]
pub struct WorldGenMountainsMapDefinition {
    pub mountains: Vec<WorldGenMountain>,
}

#[derive(SpacetimeType)]
pub struct WorldGenBuildingsMapDefinition {
    pub buildings: Vec<WorldGenBuildingDetails>,
}

#[derive(SpacetimeType)]
pub struct WorldGenResourcesMapDefinition {
    pub seed: i32,
    pub resources: Vec<WorldGenResourceDefinition>,
}

#[derive(SpacetimeType, Default, Debug)]
pub struct WorldGenAnimationCurveKeyframe {
    pub time: f32,
    pub value: f32,
    pub in_tangent: f32,
    pub out_tangent: f32,
}

#[derive(SpacetimeType)]
pub struct WorldGenLandShapeDefinition {
    pub noise_specs: WorldGenNoiseSpecs,
    pub bounds: WorldGenRectInt,
    pub land_threshold: f32,
}

#[derive(SpacetimeType)]
pub struct WorldGenBiomeDefinition {
    pub distance_to_sea_curve: WorldGenAnimationCurve,
    pub distance_to_biomes_curve: WorldGenAnimationCurve,
    pub transition_length: i32,
    pub noise_sea_multiplier: WorldGenAnimationCurve,
    pub noise_based_elevation_layers: Vec<WorldGenNoiseBasedElevationLayer>,
    pub max_lake_depth: i32,
    pub terracing: bool,
    pub grass_density: i32,
    pub lake_noise_specs: WorldGenNoiseSpecs,
    pub lake_noise_threshold: f32,
    pub lake_depth_multiplier: i32,
    pub lake_depth_smoothing: f32,
    pub lake_sea_barriers: bool,
    pub river_generation_settings: Option<WorldGenRiverGenerationSettings>,
}

#[derive(SpacetimeType)]
pub struct WorldGenMountain {
    pub center: WorldGenVector2,
    pub radius: f32,
    pub height: i32,
    pub peak_offset: WorldGenVector2,
    pub shape: WorldGenAnimationCurve,
}

#[derive(SpacetimeType)]
pub struct WorldGenBuildingDetails {
    pub index: i32,
    pub id: i32,
    pub direction: i32,
}

#[derive(SpacetimeType)]
pub struct WorldGenResourceDefinition {
    pub resource_details: WorldGenResourceDetails,
    pub biomes: Vec<WorldGenResourceBiome>,
}

#[derive(SpacetimeType, Default, Debug)]
pub struct WorldGenNoiseSpecs {
    pub seed: i32,
    pub scale: f32,
    pub octaves: i32,
    pub persistance: f32,
    pub lacunarity: f32,
}

#[derive(SpacetimeType, Default, Debug, Clone)]
pub struct WorldGenVector2 {
    pub x: f32,
    pub y: f32,
}

#[derive(SpacetimeType, Default, Debug, Clone)]
pub struct WorldGenRectInt {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(SpacetimeType)]
pub struct WorldGenResourceDetails {
    pub clump_id: i32,
    pub spawns_on_land: bool,
    pub land_elevation_range: WorldGenVector2Int,
    pub spawns_in_water: bool,
    pub water_depth_range: WorldGenVector2Int,
    pub max_elevation_delta: i32,
}

#[derive(SpacetimeType)]
pub struct WorldGenResourceBiome {
    pub biome_definition_index: i32,
    pub chance: f32,
    pub noise_threshold: WorldGenVector2,
    pub noise_specs: WorldGenNoiseSpecs,
}

#[derive(SpacetimeType)]
pub struct WorldGenRiverGenerationSettings {
    pub radius: i32,
    pub depth_curve: WorldGenAnimationCurve,
    pub erosion: f32,
    pub min_lake_circumference: i32,
    pub pathfinding_node_limit: i32,
    pub pathfinding_costs: Vec<WorldGenRiverPathfindingCosts>,
}

#[derive(SpacetimeType, Debug)]
pub struct WorldGenRiverPathfindingCosts {
    pub elevation_difference_range: WorldGenVector2Int,
    pub pathfinding_costs: f32,
}

#[derive(SpacetimeType)]
pub struct WorldGenNoiseBasedElevationLayer {
    pub blending_mode: WorldGenNoiseBasedElevationLayerBlendingMode,
    pub threshold: f32,
    pub range: WorldGenVector2Int,
    pub noise: WorldGenNoiseSpecs,
}

#[derive(SpacetimeType)]
pub enum WorldGenNoiseBasedElevationLayerBlendingMode {
    Add,
    Override,
}

#[derive(SpacetimeType, Default, Clone)]
pub struct WorldGenGeneratedResourceDeposit {
    pub x: i32,
    pub z: i32,
    pub deposit: Option<ResourceState>,
    pub dimension: u32,
}

#[derive(SpacetimeType, Default, Clone)]
pub struct WorldGenGeneratedBuilding {
    pub x: i32,
    pub z: i32,
    pub building: Option<BuildingState>,
    pub dimension: u32,
}

#[derive(SpacetimeType, Default, Clone)]
pub struct WorldGenGeneratedDroppedInventory {
    pub x: i32,
    pub z: i32,
    pub dropped_inventory: Option<DroppedInventoryState>,
    pub inventory: Option<InventoryState>,
    pub dimension: u32,
}

#[derive(SpacetimeType, Default, Clone)]
pub struct WorldGenGeneratedEnemy {
    pub x: i32,
    pub z: i32,
    pub enemy: Option<EnemyState>,
    pub dimension: u32,
}

#[derive(SpacetimeType, Default, Clone)]
pub struct WorldGenGeneratedNPC {
    pub x: i32,
    pub z: i32,
    pub npc: Option<NpcState>,
    pub dimension: u32,
}
