const MUTANT_DIVE_ENTRY_SHOT_SCREEN: Point = Point::new(0x13, 0x46);
const MUTANT_DIVE_FIRST_PATH_SHOT_SCREEN: Point = Point::new(0x1E, 0x70);
const MUTANT_DIVE_SECOND_PATH_SHOT_SCREEN: Point = Point::new(0x21, 0x87);

const MUTANT_DIVE_REFERENCE_FORCED_FIRST_PROJECTILE: MutantDiveReferenceProjectile =
    MutantDiveReferenceProjectile {
        position: Point::new(0x1E, 0x54),
        x_fraction: 0x33,
        y_fraction: 0x56,
        x_velocity: 0xFFE0,
        y_velocity: 0x0138,
    };
const MUTANT_DIVE_REFERENCE_FORCED_SECOND_PROJECTILE: MutantDiveReferenceProjectile =
    MutantDiveReferenceProjectile {
        position: Point::new(0x21, 0x7F),
        x_fraction: 0x6F,
        y_fraction: 0xE1,
        x_velocity: 0xFFF0,
        y_velocity: 0x00C0,
    };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MutantDiveReferenceProjectile {
    position: Point,
    x_fraction: u8,
    y_fraction: u8,
    x_velocity: u16,
    y_velocity: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MutantDiveReferencePathAnchor {
    world_x_word: u16,
    world_y_word: u16,
    screen: Point,
}

const MUTANT_DIVE_REFERENCE_PATH_ANCHORS: &[MutantDiveReferencePathAnchor] = &[
    MutantDiveReferencePathAnchor {
        world_x_word: 0x031C,
        world_y_word: 0x3360,
        screen: Point::new(0x12, 0x43),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x037C,
        world_y_word: 0x3380,
        screen: Point::new(0x13, 0x46),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x034C,
        world_y_word: 0x33F0,
        screen: Point::new(0x12, 0x43),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x03AC,
        world_y_word: 0x3410,
        screen: Point::new(0x14, 0x46),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x037C,
        world_y_word: 0x3480,
        screen: Point::new(0x13, 0x44),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x085C,
        world_y_word: 0x47A0,
        screen: Point::new(0x1F, 0x5B),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x085C,
        world_y_word: 0x6120,
        screen: Point::new(0x1F, 0x71),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x088C,
        world_y_word: 0x61B0,
        screen: Point::new(0x1E, 0x71),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x085C,
        world_y_word: 0x6140,
        screen: Point::new(0x1F, 0x71),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x082C,
        world_y_word: 0x7770,
        screen: Point::new(0x20, 0x87),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x07FC,
        world_y_word: 0x7800,
        screen: Point::new(0x21, 0x88),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x082C,
        world_y_word: 0x7990,
        screen: Point::new(0x20, 0x87),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x082C,
        world_y_word: 0x81E0,
        screen: Point::new(0x20, 0x90),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x082C,
        world_y_word: 0x9730,
        screen: Point::new(0x21, 0x9F),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x07FC,
        world_y_word: 0x97A0,
        screen: Point::new(0x20, 0x9E),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x085C,
        world_y_word: 0x97C0,
        screen: Point::new(0x20, 0xA0),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x088C,
        world_y_word: 0x9850,
        screen: Point::new(0x1F, 0xA0),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x085C,
        world_y_word: 0x99E0,
        screen: Point::new(0x1E, 0xA2),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x082C,
        world_y_word: 0x9A70,
        screen: Point::new(0x20, 0xA3),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x088C,
        world_y_word: 0xA200,
        screen: Point::new(0x20, 0xA2),
    },
    MutantDiveReferencePathAnchor {
        world_x_word: 0x08EC,
        world_y_word: 0xA320,
        screen: Point::new(0x20, 0xA2),
    },
];

const MUTANT_DIVE_REFERENCE_VISUAL_ROWS: &[(u16, i16)] = &[
    (0x0004, 0x36),
    (0x0034, 0x36),
    (0x0064, 0x37),
    (0x0094, 0x37),
    (0x00C4, 0x37),
    (0x00F4, 0x37),
    (0x0124, 0x36),
    (0x0154, 0x36),
    (0x0184, 0x37),
    (0x01B4, 0x37),
    (0x01E4, 0x37),
    (0x0214, 0x37),
    (0x0244, 0x36),
    (0x0274, 0x36),
    (0x02A4, 0x36),
    (0x02D4, 0x35),
    (0x0304, 0x34),
    (0x0334, 0x34),
    (0x0364, 0x32),
    (0x0394, 0x31),
    (0x03C4, 0x30),
    (0x03F4, 0x2F),
    (0x0424, 0x2F),
    (0x0454, 0x2E),
    (0x0484, 0x2D),
    (0x04B4, 0x2C),
    (0x04E4, 0x2B),
    (0x0514, 0x2C),
    (0x0544, 0x2B),
    (0x0574, 0x2B),
    (0x05A4, 0x2B),
    (0x05D4, 0x2B),
    (0x0604, 0x2A),
    (0x0634, 0x2C),
    (0x0664, 0x2C),
    (0x0694, 0x2D),
    (0x06C4, 0x2B),
    (0x06F4, 0x2B),
    (0x0724, 0x2A),
    (0x0754, 0x2C),
];
