// ── Redstone Power ──
pub const MAX_POWER: u8 = 15;
pub const WIRE_POWER_DECAY: u8 = 1;

// ── World / Chunk ──
pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_SIZE_I32: i32 = 16;
pub const WORLD_CHUNKS_X: usize = 4;
pub const WORLD_CHUNKS_Y: usize = 4;

// ── Simulation ──
pub const MAX_ITERATIONS: usize = 20;
pub const MIN_TPS: f64 = 0.15625;
pub const MAX_TPS: f64 = 81920.0;
pub const DEFAULT_TPS: f64 = 10.0;
pub const MAX_TICKS_PER_FRAME: u32 = 60;

// ── UI / Layout ──
pub const UI_BAR_HEIGHT: f32 = 60.0;
pub const PALETTE_GAP: f32 = 6.0;
pub const PALETTE_MIN_ITEM_WIDTH: f32 = 40.0;
pub const PALETTE_ITEM_INSET_Y: f32 = 5.0;
pub const PALETTE_FONT_SIZE: f32 = 10.0;
pub const PALETTE_KEY_FONT_SIZE: f32 = 9.0;
pub const PALETTE_PREVIEW_HEIGHT: f32 = 3.0;
pub const PALETTE_ITEM_HEIGHT_OFFSET: f32 = 10.0;

// ── Camera / Zoom ──
pub const CELL_SIZE: f32 = 40.0;
pub const ZOOM_STEP: f32 = 0.06;
pub const ZOOM_MIN: f32 = 0.0625;
pub const ZOOM_MAX: f32 = 16.0;
pub const ZOOM_EPSILON: f32 = 0.001;
pub const PAN_SPEED_BASE: f32 = 10.0;
pub const CENTER_CLICK_THRESHOLD: f32 = 0.25;

// ── Render / Drawing ──
pub const CULLING_MARGIN: f32 = 10.0;
pub const MIN_FONT_SIZE: f32 = 7.0;
pub const HOVER_OUTLINE_WIDTH: f32 = 2.0;
pub const SELECTION_OUTLINE_WIDTH: f32 = 2.0;
pub const TOOLTIP_FONT_SIZE: f32 = 14.0;
pub const TOOLTIP_Y_OFFSET: f32 = 22.0;
pub const COORD_FONT_SIZE: f32 = 14.0;
pub const COORD_X: f32 = 5.0;
pub const COORD_Y: f32 = 15.0;
pub const PASTE_TEXT_X: f32 = 10.0;
pub const PASTE_TEXT_Y: f32 = 20.0;
pub const PASTE_TEXT_FONT_SIZE: f32 = 18.0;
pub const STATUS_TEXT_X: f32 = 5.0;
pub const STATUS_TEXT_Y_OFFSET: f32 = 5.0;
pub const STATUS_FONT_SIZE: f32 = 12.0;
pub const SELECTION_LABEL_Y_OFFSET: f32 = 6.0;
pub const SELECTION_LABEL_FONT_SIZE: f32 = 14.0;

// ── Render: Wire ──
pub const WIRE_THIN_FRAC: f32 = 0.20;
pub const WIRE_THICK_FRAC: f32 = 0.32;
pub const WIRE_DOT_FRAC: f32 = 0.12;
pub const WIRE_TEXT_FRAC: f32 = 0.35;
pub const WIRE_TEXT_MIN: f32 = 7.0;
pub const WIRE_COLOR_R_BASE: f32 = 80.0;
pub const WIRE_COLOR_R_RANGE: f32 = 175.0;
pub const WIRE_COLOR_G_BASE: f32 = 10.0;
pub const WIRE_COLOR_G_RANGE: f32 = 30.0;
pub const WIRE_COLOR_B_BASE: f32 = 10.0;
pub const WIRE_COLOR_B_RANGE: f32 = 30.0;

// ── Render: Block outline ──
pub const BLOCK_INSET_FRAC: f32 = 0.05;
pub const BLOCK_OUTLINE_WIDTH: f32 = 1.0;

// ── Render: Torch ──
pub const TORCH_ATTACH_OFFSET_FRAC: f32 = 0.2;
pub const TORCH_STALK_W_FRAC: f32 = 0.08;
pub const TORCH_STALK_H_START_FRAC: f32 = 0.25;
pub const TORCH_STALK_H_FRAC: f32 = 0.4;
pub const TORCH_HEAD_R_FRAC: f32 = 0.16;
pub const TORCH_HEAD_INNER_R_FRAC: f32 = 0.55;
pub const TORCH_POST_W_FRAC: f32 = 0.02;
pub const TORCH_POST_H_FRAC: f32 = 0.5;
pub const TORCH_FLOOR_STALK_W_FRAC: f32 = 0.1;
pub const TORCH_FLOOR_STALK_H_FRAC: f32 = 0.5;
pub const TORCH_FLOOR_HEAD_R_FRAC: f32 = 0.18;
pub const TORCH_FLOOR_GLOW_R_FRAC: f32 = 0.6;

// ── Render: Repeater ──
pub const REPEATER_ARROW_LEN_FRAC: f32 = 0.3;
pub const REPEATER_LINE_W_FRAC: f32 = 0.08;
pub const REPEATER_PERP_FRAC: f32 = 0.07;
pub const REPEATER_ARROW_W_FRAC: f32 = 0.06;
pub const REPEATER_TEXT_X_FRAC: f32 = 0.35;
pub const REPEATER_TEXT_Y_FRAC: f32 = 0.85;
pub const REPEATER_TEXT_MIN: f32 = 7.0;
pub const REPEATER_POWER_X_FRAC: f32 = 0.05;
pub const REPEATER_POWER_Y_FRAC: f32 = 0.35;
pub const REPEATER_POWER_TEXT_FRAC: f32 = 0.3;

// ── Render: Comparator ──
pub const COMPARATOR_ARROW_LEN_FRAC: f32 = 0.25;
pub const COMPARATOR_LINE_W_FRAC: f32 = 0.08;
pub const COMPARATOR_CIRCLE_OFF_FRAC: f32 = 0.15;
pub const COMPARATOR_CIRCLE_R_FRAC: f32 = 0.08;
pub const COMPARATOR_TEXT_X_FRAC: f32 = 0.35;
pub const COMPARATOR_TEXT_Y_FRAC: f32 = 0.85;
pub const COMPARATOR_TEXT_MIN: f32 = 8.0;
pub const COMPARATOR_POWER_TEXT_FRAC: f32 = 0.25;
pub const COMPARATOR_POWER_X_FRAC: f32 = 0.05;
pub const COMPARATOR_POWER_Y_FRAC: f32 = 0.3;
pub const COMPARATOR_POWER_MIN: f32 = 6.0;

// ── Render: Lever ──
pub const LEVER_STICK_LEN_FRAC: f32 = 0.35;
pub const LEVER_LINE_W_FRAC: f32 = 0.07;
pub const LEVER_KNOB_R_FRAC: f32 = 0.12;

// ── Render: Button ──
pub const BUTTON_INNER_FRAC: f32 = 0.15;

// ── Render: Target ──
pub const TARGET_RING_R_FRAC: f32 = 0.3;
pub const TARGET_RING_W_FRAC: f32 = 0.06;
pub const TARGET_DOT_R_FRAC: f32 = 0.08;

// ── Render: Barrel ──
pub const BARREL_STRIPE_X_FRAC: f32 = 0.15;
pub const BARREL_STRIPE_Y_FRAC: f32 = 0.1;
pub const BARREL_STRIPE_W_FRAC: f32 = 0.7;
pub const BARREL_STRIPE_H_FRAC: f32 = 0.12;
pub const BARREL_STRIPE2_Y_FRAC: f32 = 0.78;
pub const BARREL_TEXT_FRAC: f32 = 0.4;
pub const BARREL_TEXT_MIN: f32 = 9.0;

// ── Render: Solid block power text ──
pub const SOLID_POWER_TEXT_FRAC: f32 = 0.3;
pub const SOLID_POWER_TEXT_MIN: f32 = 7.0;

// ── Colors (RGBA tuples) ──
pub const BG_CLEAR_COLOR: (u8, u8, u8, u8) = (20, 20, 20, 255);
pub const HOVER_HIGHLIGHT_COLOR: (u8, u8, u8, u8) = (255, 255, 255, 100);
pub const SELECTION_FILL_COLOR: (u8, u8, u8, u8) = (255, 255, 255, 20);
pub const SELECTION_OUTLINE_COLOR: (u8, u8, u8, u8) = (255, 255, 255, 160);
pub const SELECTION_LABEL_COLOR: (u8, u8, u8, u8) = (255, 255, 255, 200);
pub const PASTE_MODE_TEXT_COLOR: (u8, u8, u8, u8) = (255, 255, 100, 255);
pub const STATUS_TEXT_COLOR: (u8, u8, u8, u8) = (150, 150, 150, 255);
pub const UNLOADED_CHUNK_COLOR: (u8, u8, u8, u8) = (18, 18, 22, 255);
pub const CHECKER_EVEN_COLOR: (u8, u8, u8, u8) = (35, 35, 35, 255);
pub const CHECKER_ODD_COLOR: (u8, u8, u8, u8) = (28, 28, 28, 255);
pub const CHUNK_GRID_COLOR: (u8, u8, u8, u8) = (50, 50, 55, 255);
pub const BAR_BG_COLOR: (u8, u8, u8, u8) = (35, 35, 40, 255);
pub const BAR_LINE_COLOR: (u8, u8, u8, u8) = (70, 70, 80, 255);
pub const PALETTE_SELECTED_COLOR: (u8, u8, u8, u8) = (70, 80, 120, 255);
pub const PALETTE_UNSELECTED_COLOR: (u8, u8, u8, u8) = (45, 45, 50, 255);
pub const PALETTE_TEXT_COLOR: (u8, u8, u8, u8) = (180, 180, 180, 255);
pub const PALETTE_KEY_COLOR: (u8, u8, u8, u8) = (120, 120, 140, 200);
pub const TOOLTIP_BG_COLOR: (u8, u8, u8, u8) = (0, 0, 0, 200);
pub const COORD_TEXT_COLOR: (u8, u8, u8, u8) = (180, 180, 180, 255);
pub const SOLID_BLOCK_FILL_COLOR: (u8, u8, u8, u8) = (110, 110, 120, 255);
pub const SOLID_BLOCK_OUTLINE_COLOR: (u8, u8, u8, u8) = (140, 140, 150, 255);
pub const SOLID_POWER_COLOR: (u8, u8, u8, u8) = (255, 200, 100, 220);
pub const REDSTONE_BLOCK_FILL_COLOR: (u8, u8, u8, u8) = (180, 25, 25, 255);
pub const REDSTONE_BLOCK_OUTLINE_COLOR: (u8, u8, u8, u8) = (210, 50, 50, 255);
pub const REPEATER_OFF_COLOR: (u8, u8, u8, u8) = (130, 120, 110, 255);
pub const REPEATER_ON_COLOR: (u8, u8, u8, u8) = (200, 80, 80, 255);
pub const REPEATER_POWER_COLOR: (u8, u8, u8, u8) = (255, 200, 100, 200);
pub const COMPARATOR_OFF_COLOR: (u8, u8, u8, u8) = (130, 130, 140, 255);
pub const COMPARATOR_ON_COLOR: (u8, u8, u8, u8) = (200, 100, 60, 255);
pub const COMPARATOR_COMPARE_COLOR: (u8, u8, u8, u8) = (100, 200, 255, 255);
pub const COMPARATOR_SUBTRACT_COLOR: (u8, u8, u8, u8) = (255, 200, 100, 255);
pub const COMPARATOR_POWER_COLOR: (u8, u8, u8, u8) = (255, 220, 150, 200);
pub const LEVER_BG_COLOR: (u8, u8, u8, u8) = (70, 70, 70, 255);
pub const LEVER_STICK_COLOR: (u8, u8, u8, u8) = (140, 100, 60, 255);
pub const LEVER_KNOB_ON_COLOR: (u8, u8, u8, u8) = (255, 50, 50, 255);
pub const LEVER_KNOB_OFF_COLOR: (u8, u8, u8, u8) = (100, 100, 100, 255);
pub const BUTTON_ON_COLOR: (u8, u8, u8, u8) = (200, 200, 80, 255);
pub const BUTTON_OFF_COLOR: (u8, u8, u8, u8) = (100, 100, 80, 255);
pub const BUTTON_INNER_COLOR: (u8, u8, u8, u8) = (60, 60, 60, 255);
pub const LAMP_ON_COLOR: (u8, u8, u8, u8) = (255, 255, 120, 255);
pub const LAMP_OFF_COLOR: (u8, u8, u8, u8) = (80, 80, 40, 255);
pub const LAMP_ON_OUTLINE_COLOR: (u8, u8, u8, u8) = (255, 255, 180, 200);
pub const TARGET_FILL_COLOR: (u8, u8, u8, u8) = (180, 100, 100, 255);
pub const TARGET_RING_COLOR: (u8, u8, u8, u8) = (120, 60, 60, 255);
pub const TARGET_DOT_COLOR: (u8, u8, u8, u8) = (60, 30, 30, 255);
pub const TARGET_OUTLINE_COLOR: (u8, u8, u8, u8) = (200, 120, 120, 255);
pub const BARREL_FILL_COLOR: (u8, u8, u8, u8) = (140, 100, 70, 255);
pub const BARREL_STRIPE_COLOR: (u8, u8, u8, u8) = (100, 70, 50, 255);
pub const BARREL_TEXT_COLOR: (u8, u8, u8, u8) = (255, 220, 150, 220);
pub const TORCH_BODY_LIT_COLOR: (u8, u8, u8, u8) = (255, 130, 20, 255);
pub const TORCH_BODY_UNLIT_COLOR: (u8, u8, u8, u8) = (80, 40, 0, 255);
pub const TORCH_GLOW_COLOR: (u8, u8, u8, u8) = (255, 220, 80, 200);
pub const TORCH_POST_COLOR: (u8, u8, u8, u8) = (50, 30, 20, 255);

// ── Palette swatch ──
pub const PALETTE_SWATCH_FRAC: f32 = 0.5;
pub const PALETTE_SWATCH_HEIGHT_FRAC: f32 = 0.45;
pub const PALETTE_SWATCH_Y_OFFSET: f32 = 3.0;
pub const PALETTE_SWATCH_H_FRAC: f32 = 0.7;

// ── Filesystem ──
pub const TEMP_SAVE_PATH: &str = "saves/temp.json";
pub const DEFAULT_SAVE_DIR: &str = "saves\\";
pub const SAVE_EXTENSION: &str = "*.json";
pub const SAVE_EXTENSION_LABEL: &str = "JSON files";
pub const NBT_EXTENSION: &str = "*.nbt";
pub const NBT_EXTENSION_LABEL: &str = "NBT files";

// ── Block Data Encoding (bit-field layout) ──
// Direction: bits [1:0]
pub const DIR_MASK: u16 = 0x3;

// Repeater: delay[3:2], locked[4], powered[5], counter[8:6]
pub const REPEATER_DELAY_SHIFT: u16 = 2;
pub const REPEATER_DELAY_MASK: u8 = 3;
pub const REPEATER_MAX_DELAY: u8 = 3;
pub const REPEATER_LOCKED_SHIFT: u16 = 4;
pub const REPEATER_POWERED_SHIFT: u16 = 5;
pub const REPEATER_COUNTER_SHIFT: u16 = 6;
pub const REPEATER_COUNTER_MASK: u8 = 7;
pub const REPEATER_MAX_COUNTER: u8 = 7;
pub const REPEATER_DELAY_COUNT: u8 = 4;

// Comparator: mode[2], powered[3]
pub const COMPARATOR_MODE_SHIFT: u16 = 2;
pub const COMPARATOR_POWERED_SHIFT: u16 = 3;

// Lever: powered[2]
pub const LEVER_POWERED_SHIFT: u16 = 2;

// Torch: lit[0], on_wall[1], dir[3:2]
pub const TORCH_DIR_SHIFT: u16 = 2;

// Lamp: lit[0]

// Button: powered[2], counter[7:3]
pub const BUTTON_POWERED_SHIFT: u16 = 2;
pub const BUTTON_COUNTER_SHIFT: u16 = 3;
pub const BUTTON_COUNTER_MASK: u8 = 0x1F;
pub const BUTTON_MAX_COUNTER: u8 = 31;
pub const BUTTON_DEFAULT_TICKS: u8 = 20;

// Barrel: strength[3:0]
pub const BARREL_STRENGTH_MASK: u16 = 0xF;
pub const BARREL_MAX_STRENGTH: u8 = 15;
pub const BARREL_STRENGTH_COUNT: u8 = 16;

// ── Misc ──
pub const TEST_CIRCUIT_OFFSET: i32 = 3;
