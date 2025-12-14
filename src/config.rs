// Configuration file, All measurements in real-world meters (1 unit = 1 meter)
// This controls the initial generation parameter settings

// Current parameter settings are based on values as documented in my report

// Point generation boundaries (meters)
pub const CANVAS_WIDTH: f32 = 500.0;  // Working area width; not settlement size
pub const CANVAS_HEIGHT: f32 = 500.0; // Working area height; not settlement size

pub const INITIAL_SEED: u64 = 1512086461918454205;
pub const POINT_COUNT: usize = 30;
pub const SPIRAL_SPREAD: f32 = 3.0;  // adjust initial point distribution here

// Building parameters (from morphological analysis)
pub const BUILDING_AREA_MIN: f32 = 15.0;  // Minimum building footprint area (m2)
pub const BUILDING_AREA_MAX: f32 = 40.0;  // Maximum building footprint area (m2)

// Subdivision control parameters
pub const CIRCUMCENTER_MERGE_THRESHOLD: f32 = 0.01;  // merge circumcenters closer than this distance

// Morphological variation parameters (dimensionless ratios)
pub const GRID_CHAOS: f32 = 0.35;     // Geometric irregularity factor
pub const SIZE_CHAOS: f32 = 0.25;     // Building size variation factor  
pub const EMPTY_PROB: f32 = 0.05;     // Probability of a plot being empty

// Settlement boundary parameters
pub const BOUNDARY_GENERATOR_SPACING: f32 = 12.0;       // Generator spacing along boundary edges
pub const BOUNDARY_GENERATOR_INNER_OFFSET: f32 = 1.0;   // Inner boundary generator offset
pub const BOUNDARY_GENERATOR_OUTER_OFFSET: f32 = 2.0;   // Outer boundary generator offset

// Subdivision parameters
pub const MAX_RECURSION_DEPTH: usize = 10;

// Alley parameters
pub const ALLEY_WIDTH_MIN: f32 = 0.5;   // Minimum alley width
pub const ALLEY_WIDTH_MAX: f32 = 1.5;   // Maximum alley width 
pub const ALLEY_WIDTH: f32 = 0.8;       // Default alley width
pub const ALLEY_CHANCE: f32 = 0.8;      // Probability of creating alleys

// Road constraint parameters
pub const ROAD_GENERATOR_SPACING: f32 = 7.0;   // Generator spacing along roads
pub const ROAD_GENERATOR_OFFSET: f32 = 0.1;    // Road generator offset
pub const CORNER_CONSTRAINT_DISTANCE: f32 = 2.0; // Corner constraint distance
pub const ROAD_WIDTH: f32 = 4.0; // Road corridor width

// 3D building parameters, these are custom
pub const MIN_WALL_HEIGHT: f32 = 2.0;   // Minimum wall height
pub const MAX_WALL_HEIGHT: f32 = 6.0;   // Maximum wall height
pub const MIN_ROOF_HEIGHT: f32 = 0.7;   // Minimum roof height
pub const MAX_ROOF_HEIGHT: f32 = 1.0;   // Maximum roof height

// roof heights are currently deprecated, 
// I used to use them for moving the roof centroid up to make pyramids