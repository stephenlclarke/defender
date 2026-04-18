//! Renders Defender scenes into RGBA frames for Kitty graphics output and README media.

use crate::{
    font::arcade_font,
    game::{Entity, EntityKind, HorizontalDirection, World},
    high_scores::{HighScoreEntry, HighScoreTable},
    render::InitialsEntryView,
    sprites::arcade_sprites,
    terminal::TerminalGeometry,
};

const LOGICAL_WIDTH: u32 = 960;
const LOGICAL_HEIGHT: u32 = 720;
const MAX_RENDER_WIDTH: u32 = 1_280;
const MAX_RENDER_HEIGHT: u32 = 960;
const BACKGROUND: [u8; 4] = [2, 5, 11, 255];
const PANEL_BACKGROUND: [u8; 4] = [10, 14, 26, 255];
const PANEL_BORDER: [u8; 4] = [86, 123, 255, 255];
const TEXT_PRIMARY: [u8; 4] = [240, 244, 255, 255];
const TEXT_SECONDARY: [u8; 4] = [130, 212, 255, 255];
const TEXT_WARNING: [u8; 4] = [255, 200, 88, 255];
const TEXT_DANGER: [u8; 4] = [255, 96, 88, 255];
const TEXT_SCORE_BLUE: [u8; 4] = [84, 196, 255, 255];
const TEXT_ARCADE_WHITE: [u8; 4] = [246, 246, 246, 255];
const TERRAIN_LINE: [u8; 4] = [72, 224, 96, 255];
const TERRAIN_FILL: [u8; 4] = [11, 50, 22, 255];
const TERRAIN_AMBER_LINE: [u8; 4] = [255, 164, 40, 255];
const TERRAIN_AMBER_FILL: [u8; 4] = [72, 40, 0, 255];
const VIEWPORT_BORDER: [u8; 4] = [68, 94, 180, 255];
const VIEWPORT_BACKGROUND: [u8; 4] = [0, 0, 0, 255];
const SCANNER_BACKGROUND: [u8; 4] = [5, 8, 18, 255];
const SCANNER_BORDER: [u8; 4] = [67, 114, 198, 255];
const PLAYER_COLOR: [u8; 4] = [255, 255, 255, 255];
const HUMAN_COLOR: [u8; 4] = [178, 255, 160, 255];
const LANDER_COLOR: [u8; 4] = [255, 132, 92, 255];
const MUTANT_COLOR: [u8; 4] = [255, 88, 160, 255];
const BAITER_COLOR: [u8; 4] = [255, 220, 84, 255];
const BOMBER_COLOR: [u8; 4] = [108, 224, 255, 255];
const POD_COLOR: [u8; 4] = [190, 120, 255, 255];
const SWARMER_COLOR: [u8; 4] = [255, 170, 92, 255];
const ENEMY_SHOT_COLOR: [u8; 4] = [255, 94, 94, 255];
const PLAYER_SHOT_COLOR: [u8; 4] = [255, 255, 140, 255];
const MINE_COLOR: [u8; 4] = [255, 74, 34, 255];
const ATTRACT_SCORE_CARD: [(&str, u32); 6] = [
    ("LANDER", 150),
    ("MUTANT", 150),
    ("BAITER", 200),
    ("BOMBER", 250),
    ("POD", 1000),
    ("SWARMER", 150),
];

pub enum Screen<'a> {
    Logo,
    Title {
        high_score: u32,
        xyzzy_active: bool,
        invincible: bool,
        auto_fire: bool,
    },
    Attract {
        world: &'a World,
        revealed_score_entries: usize,
    },
    HighScores {
        todays: &'a HighScoreTable,
        all_time: &'a HighScoreTable,
    },
    Playing {
        world: &'a World,
        xyzzy_active: bool,
        invincible: bool,
        auto_fire: bool,
    },
    GameOver {
        world: &'a World,
        high_score: u32,
        xyzzy_active: bool,
        invincible: bool,
        auto_fire: bool,
    },
    InitialsEntry {
        world: &'a World,
        view: &'a InitialsEntryView<'a>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub struct Renderer {
    image_width: u32,
    image_height: u32,
    render_target: RenderedImage,
}

#[derive(Clone, Copy)]
struct Color(u8, u8, u8, u8);

#[derive(Clone, Copy)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Renderer {
    pub fn new(geometry: TerminalGeometry) -> Self {
        let (image_width, image_height) = raster_size(geometry);
        Self::with_size(image_width, image_height)
    }

    pub fn with_size(image_width: u32, image_height: u32) -> Self {
        Self {
            image_width,
            image_height,
            render_target: RenderedImage::new_blank(image_width, image_height, BACKGROUND),
        }
    }

    pub fn resize(&mut self, geometry: TerminalGeometry) {
        let (image_width, image_height) = raster_size(geometry);
        self.image_width = image_width;
        self.image_height = image_height;
        self.render_target
            .resize(image_width, image_height, BACKGROUND);
    }

    pub fn image_width(&self) -> u32 {
        self.image_width
    }

    pub fn image_height(&self) -> u32 {
        self.image_height
    }

    pub fn render(&mut self, screen: Screen<'_>) -> &RenderedImage {
        self.render_target.clear(Color::from_rgba(BACKGROUND));

        match screen {
            Screen::Logo => self.render_logo_screen(),
            Screen::Title {
                high_score,
                xyzzy_active,
                invincible,
                auto_fire,
            } => self.render_title_screen(high_score, xyzzy_active, invincible, auto_fire),
            Screen::Attract {
                world,
                revealed_score_entries,
            } => self.render_attract_screen(world, revealed_score_entries),
            Screen::HighScores { todays, all_time } => {
                self.render_high_scores_screen(todays, all_time)
            }
            Screen::Playing {
                world,
                xyzzy_active,
                invincible,
                auto_fire,
            } => self.render_playing_screen(world, xyzzy_active, invincible, auto_fire),
            Screen::GameOver {
                world,
                high_score,
                xyzzy_active,
                invincible,
                auto_fire,
            } => {
                self.render_game_over_screen(world, high_score, xyzzy_active, invincible, auto_fire)
            }
            Screen::InitialsEntry { world, view } => self.render_initials_screen(world, view),
        }

        &self.render_target
    }

    fn render_logo_screen(&mut self) {
        self.draw_space_backdrop(0, None);
        self.draw_centered_text(self.image_width as i32 / 2, 96, "WILLIAMS", TEXT_WARNING, 4);
        self.draw_centered_text(
            self.image_width as i32 / 2,
            146,
            "ELECTRONICS INC.",
            TEXT_WARNING,
            2,
        );
        self.draw_centered_text(
            self.image_width as i32 / 2,
            214,
            "PRESENTS",
            TEXT_SECONDARY,
            2,
        );
        self.draw_title_logo(self.image_width as i32 / 2, 370);
        self.draw_centered_text(
            self.image_width as i32 / 2,
            self.image_height as i32 - 78,
            "COPYRIGHT 1980 - WILLIAMS ELECTRONICS",
            TEXT_SECONDARY,
            2,
        );
    }

    fn render_title_screen(
        &mut self,
        high_score: u32,
        xyzzy_active: bool,
        invincible: bool,
        auto_fire: bool,
    ) {
        self.draw_space_backdrop(3, None);
        self.draw_title_logo(self.image_width as i32 / 2, 210);
        self.draw_centered_text(
            self.image_width as i32 / 2,
            334,
            &format!("HIGH SCORE TO DATE {:05}", high_score),
            TEXT_PRIMARY,
            3,
        );
        self.draw_centered_text(
            self.image_width as i32 / 2,
            404,
            "PRESS ENTER OR 1 PLAYER START",
            TEXT_WARNING,
            3,
        );
        self.draw_centered_text(
            self.image_width as i32 / 2,
            452,
            "PRESS Q OR ESC TO QUIT",
            TEXT_SECONDARY,
            2,
        );

        let controls_panel = Rect {
            x: 104,
            y: 502,
            width: self.image_width as i32 - 208,
            height: 170,
        };
        self.draw_panel(controls_panel);
        self.draw_text(
            controls_panel.x + 24,
            controls_panel.y + 20,
            "CONTROLS",
            TEXT_WARNING,
            2,
        );
        let lines = [
            "A / Z      MOVE UP / DOWN",
            "SHIFT      THRUST",
            "SPACE      FLIP DIRECTION",
            "ENTER      FIRE",
            "TAB        SMART BOMB",
            "H          HYPERSPACE",
        ];
        for (index, line) in lines.into_iter().enumerate() {
            self.draw_text(
                controls_panel.x + 24,
                controls_panel.y + 52 + index as i32 * 18,
                line,
                TEXT_PRIMARY,
                2,
            );
        }

        self.draw_secret_status(xyzzy_active, invincible, auto_fire, 24, 26);
    }

    fn render_attract_screen(&mut self, world: &World, revealed_score_entries: usize) {
        let sidebar = Rect {
            x: self.image_width as i32 - 248,
            y: 122,
            width: 200,
            height: self.image_height as i32 - 168,
        };
        let playfield = Rect {
            x: 36,
            y: 122,
            width: self.image_width as i32 - sidebar.width - 96,
            height: self.image_height as i32 - 168,
        };

        self.draw_space_backdrop(world.tick(), Some(playfield));
        self.draw_centered_text(
            self.image_width as i32 / 2,
            40,
            "PRESS 1 OR 2 PLAYER START",
            TEXT_WARNING,
            3,
        );
        self.draw_centered_text(
            self.image_width as i32 / 2,
            78,
            "DEFENDER ATTRACT MODE",
            TEXT_SECONDARY,
            2,
        );
        self.draw_world_panel(world, playfield, false);

        self.draw_panel(sidebar);
        self.draw_text(sidebar.x + 24, sidebar.y + 18, "SCANNER", TEXT_WARNING, 2);
        self.draw_text(
            sidebar.x + 24,
            sidebar.y + 50,
            "TARGET VALUES",
            TEXT_PRIMARY,
            2,
        );
        for (index, (name, score)) in ATTRACT_SCORE_CARD
            .into_iter()
            .take(revealed_score_entries)
            .enumerate()
        {
            self.draw_text(
                sidebar.x + 24,
                sidebar.y + 92 + index as i32 * 42,
                name,
                color_for_enemy_name(name),
                2,
            );
            self.draw_text(
                sidebar.x + 24,
                sidebar.y + 110 + index as i32 * 42,
                &format!("{score:>4}"),
                TEXT_PRIMARY,
                3,
            );
        }
    }

    fn render_high_scores_screen(&mut self, todays: &HighScoreTable, all_time: &HighScoreTable) {
        self.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width: self.image_width as i32,
                height: self.image_height as i32,
            },
            Color::from_rgba(BACKGROUND),
        );
        self.draw_arcade_logo(self.image_width as i32 / 2, 66, 5);
        self.draw_centered_text(
            self.image_width as i32 / 2,
            142,
            "HALL OF FAME",
            TEXT_SCORE_BLUE,
            2,
        );
        self.draw_score_tables(
            Rect {
                x: 96,
                y: 186,
                width: self.image_width as i32 - 192,
                height: self.image_height as i32 - 236,
            },
            todays,
            all_time,
        );
    }

    fn render_playing_screen(
        &mut self,
        world: &World,
        xyzzy_active: bool,
        invincible: bool,
        auto_fire: bool,
    ) {
        let playfield = self.default_playfield_rect();
        self.draw_space_backdrop(world.tick(), Some(playfield));
        self.draw_hud(world);
        self.draw_scanner(world, self.default_scanner_rect());
        self.draw_world_panel(world, playfield, true);
        self.draw_secret_status(
            xyzzy_active,
            invincible,
            auto_fire,
            24,
            self.image_height as i32 - 84,
        );
    }

    fn render_game_over_screen(
        &mut self,
        world: &World,
        _high_score: u32,
        _xyzzy_active: bool,
        _invincible: bool,
        _auto_fire: bool,
    ) {
        self.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width: self.image_width as i32,
                height: self.image_height as i32,
            },
            Color::from_rgba(BACKGROUND),
        );
        let strip_y = 82;
        self.draw_text(
            36,
            28,
            &format!("{:>5}", world.status().score),
            TEXT_ARCADE_WHITE,
            2,
        );
        self.draw_arcade_game_over_scanner(
            world,
            Rect {
                x: self.image_width as i32 / 2 - 160,
                y: 8,
                width: 320,
                height: 60,
            },
        );
        self.draw_line(
            0,
            strip_y,
            self.image_width as i32,
            strip_y,
            Color::from_rgba(TERRAIN_LINE),
            2,
        );

        let playfield = Rect {
            x: 0,
            y: strip_y + 4,
            width: self.image_width as i32,
            height: self.image_height as i32 - strip_y - 4,
        };
        self.draw_world_panel_with_style(
            world,
            playfield,
            false,
            None,
            TERRAIN_AMBER_FILL,
            TERRAIN_AMBER_LINE,
        );
        self.draw_centered_text(
            self.image_width as i32 / 2,
            playfield.y + playfield.height / 2 - 18,
            "GAME OVER",
            TEXT_ARCADE_WHITE,
            3,
        );
    }

    fn render_initials_screen(&mut self, world: &World, view: &InitialsEntryView<'_>) {
        self.draw_space_backdrop(world.tick(), Some(self.default_playfield_rect()));
        self.draw_world_panel(world, self.default_playfield_rect(), true);
        self.draw_scanner(world, self.default_scanner_rect());

        let panel = Rect {
            x: 92,
            y: 116,
            width: self.image_width as i32 - 184,
            height: self.image_height as i32 - 168,
        };
        self.draw_panel(panel);
        self.draw_centered_text(
            panel.center_x(),
            panel.y + 22,
            "DEFENDER HALL OF FAME",
            TEXT_WARNING,
            3,
        );
        self.draw_centered_text(
            panel.center_x(),
            panel.y + 66,
            &format!(
                "SCORE {:06}   HIGH SCORE {:05}",
                view.entry_score, view.high_score
            ),
            TEXT_PRIMARY,
            2,
        );
        self.draw_centered_text(
            panel.center_x(),
            panel.y + 92,
            &format!("QUALIFIES FOR RANK {:>2}", view.entry_rank),
            TEXT_SECONDARY,
            2,
        );
        self.draw_centered_text(
            panel.center_x(),
            panel.y + 126,
            "ENTER INITIALS",
            TEXT_WARNING,
            2,
        );
        self.draw_centered_text(
            panel.center_x(),
            panel.y + 156,
            &format!("[{}]", view.initials),
            TEXT_PRIMARY,
            4,
        );
        self.draw_centered_text(
            panel.center_x(),
            panel.y + 196,
            "TYPE LETTERS A-Z  BACKSPACE DELETES  ENTER SAVES",
            TEXT_SECONDARY,
            1,
        );
        self.draw_score_tables(
            Rect {
                x: panel.x + 28,
                y: panel.y + 240,
                width: panel.width - 56,
                height: panel.height - 268,
            },
            view.todays_high_scores,
            view.all_time_high_scores,
        );
        self.draw_secret_status(
            view.xyzzy_active,
            view.invincible,
            view.auto_fire,
            panel.x + 24,
            panel.y + panel.height - 52,
        );
    }

    fn draw_hud(&mut self, world: &World) {
        self.draw_text(24, 18, "DEFENDER", TEXT_WARNING, 3);
        self.draw_text(
            262,
            20,
            &format!("SCORE {:06}", world.status().score),
            TEXT_PRIMARY,
            2,
        );
        self.draw_text(
            500,
            20,
            &format!("LIVES {}", world.status().lives),
            TEXT_PRIMARY,
            2,
        );
        self.draw_text(
            642,
            20,
            &format!("WAVE {}", world.status().wave),
            TEXT_PRIMARY,
            2,
        );
        self.draw_text(
            760,
            20,
            &format!("BOMBS {}", world.smart_bombs()),
            TEXT_PRIMARY,
            2,
        );
        self.draw_text(
            24,
            52,
            &format!(
                "ENEMIES {}   HUMANS {}   THREAT {}",
                world.enemy_count(),
                world.human_count(),
                world.threat_score()
            ),
            TEXT_SECONDARY,
            2,
        );
        self.draw_text(
            642,
            52,
            if world.planet_destroyed() {
                "DEEP SPACE"
            } else {
                "PLANET ACTIVE"
            },
            if world.planet_destroyed() {
                TEXT_DANGER
            } else {
                TEXT_SECONDARY
            },
            2,
        );
    }

    fn draw_scanner(&mut self, world: &World, rect: Rect) {
        self.fill_rect(rect, Color::from_rgba(SCANNER_BACKGROUND));
        self.stroke_rect(rect, Color::from_rgba(SCANNER_BORDER), 2);
        self.draw_text(rect.x + 12, rect.y + 10, "SCANNER", TEXT_SECONDARY, 1);
        let band = Rect {
            x: rect.x + 86,
            y: rect.y + 10,
            width: rect.width - 100,
            height: rect.height - 20,
        };
        self.fill_rect(band, Color::from_rgba([4, 6, 14, 255]));
        self.stroke_rect(band, Color::from_rgba([28, 48, 88, 255]), 1);

        for entity in world.entities() {
            let index = ((entity.position.x.rem_euclid(world.world_span()) as usize)
                * band.width as usize)
                / world.world_span() as usize;
            let x = band.x + index as i32;
            let y = band.center_y();
            let color = scanner_color(entity.kind);
            let radius = if entity.kind == EntityKind::PlayerShip {
                3
            } else {
                2
            };
            self.draw_dot(x, y, Color::from_rgba(color), radius);
        }
    }

    fn draw_arcade_game_over_scanner(&mut self, world: &World, rect: Rect) {
        self.stroke_rect(rect, Color::from_rgba(TERRAIN_LINE), 2);
        let inner = rect.inset(2);
        self.fill_rect(inner, Color::from_rgba(BACKGROUND));

        if !world.planet_destroyed() {
            let terrain_y = inner.y + inner.height - 10;
            let mut previous = None;
            for screen_x in 0..world.width() {
                let x = inner.x
                    + ((screen_x as f32 + 0.5) * inner.width as f32 / world.width() as f32).round()
                        as i32;
                let terrain_row = world.terrain_row_at_screen_x(screen_x) as f32;
                let terrain_offset =
                    ((terrain_row / world.height() as f32) * 12.0).round() as i32 - 6;
                let y = terrain_y + terrain_offset;
                if let Some((prev_x, prev_y)) = previous {
                    self.draw_line(
                        prev_x,
                        prev_y,
                        x,
                        y,
                        Color::from_rgba(TERRAIN_AMBER_LINE),
                        1,
                    );
                }
                previous = Some((x, y));
            }
        }

        for entity in world.entities() {
            let x = inner.x
                + ((entity.position.x.rem_euclid(world.world_span()) as f32) * inner.width as f32
                    / world.world_span() as f32)
                    .round() as i32;
            let y = inner.y
                + ((entity.position.y as f32 + 0.5) * inner.height as f32 / world.height() as f32)
                    .round() as i32;
            let radius = if entity.kind == EntityKind::PlayerShip {
                3
            } else {
                2
            };
            self.draw_dot(x, y, Color::from_rgba(scanner_color(entity.kind)), radius);
        }
    }

    fn draw_world_panel(&mut self, world: &World, rect: Rect, show_status_overlay: bool) {
        self.draw_world_panel_with_style(
            world,
            rect,
            show_status_overlay,
            Some(VIEWPORT_BORDER),
            TERRAIN_FILL,
            TERRAIN_LINE,
        );
    }

    fn draw_world_panel_with_style(
        &mut self,
        world: &World,
        rect: Rect,
        show_status_overlay: bool,
        border_color: Option<[u8; 4]>,
        terrain_fill_color: [u8; 4],
        terrain_line_color: [u8; 4],
    ) {
        self.fill_rect(rect, Color::from_rgba(VIEWPORT_BACKGROUND));
        if let Some(border_color) = border_color {
            self.stroke_rect(rect, Color::from_rgba(border_color), 2);
        }
        let terrain_rect = if border_color.is_some() {
            rect.inset(2)
        } else {
            rect
        };
        self.draw_space_backdrop(world.tick().wrapping_add(17), Some(terrain_rect));

        if !world.planet_destroyed() {
            let mut previous = None;
            for screen_x in 0..world.width() {
                let x = terrain_rect.x
                    + ((screen_x as f32 + 0.5) * terrain_rect.width as f32 / world.width() as f32)
                        .round() as i32;
                let y = terrain_rect.y
                    + ((world.terrain_row_at_screen_x(screen_x) as f32 + 0.5)
                        * terrain_rect.height as f32
                        / world.height() as f32)
                        .round() as i32;
                self.draw_line(
                    x,
                    y,
                    x,
                    terrain_rect.y + terrain_rect.height - 1,
                    Color::from_rgba(terrain_fill_color),
                    1,
                );
                if let Some((prev_x, prev_y)) = previous {
                    self.draw_line(
                        prev_x,
                        prev_y,
                        x,
                        y,
                        Color::from_rgba(terrain_line_color),
                        2,
                    );
                }
                previous = Some((x, y));
            }
        }

        let cell_width = (terrain_rect.width as f32 / world.width() as f32).max(4.0);
        let cell_height = (terrain_rect.height as f32 / world.height() as f32).max(4.0);

        for entity in world.entities() {
            if let Some(screen_x) = world.screen_x_for_world_x(entity.position.x) {
                let cx = terrain_rect.x + ((screen_x as f32 + 0.5) * cell_width).round() as i32;
                let cy = terrain_rect.y
                    + ((entity.position.y as f32 + 0.5) * cell_height).round() as i32;
                let scale = cell_width.min(cell_height).round() as i32;
                self.draw_entity(
                    entity,
                    world.player_facing(),
                    world.tick(),
                    cx,
                    cy,
                    scale.max(4),
                );
            }
        }

        if show_status_overlay {
            self.draw_text(
                rect.x + 14,
                rect.y + 12,
                &format!("CAM {:03}   TICK {:03}", world.camera_x(), world.tick()),
                TEXT_SECONDARY,
                1,
            );
        }
    }

    fn draw_entity(
        &mut self,
        entity: &Entity,
        facing: HorizontalDirection,
        tick: u32,
        cx: i32,
        cy: i32,
        scale: i32,
    ) {
        let sprites = arcade_sprites();
        let image = sprites.sprite_for_entity(entity, tick, facing);
        self.draw_scaled_image_centered(
            image.as_ref(),
            cx,
            cy,
            sprite_draw_height(entity.kind, scale),
        );
    }

    fn draw_score_tables(
        &mut self,
        rect: Rect,
        todays: &HighScoreTable,
        all_time: &HighScoreTable,
    ) {
        let left_center = rect.x + rect.width / 4;
        let right_center = rect.x + rect.width * 3 / 4;
        let table_top = rect.y + 6;
        self.draw_centered_text(left_center, table_top, "TODAYS", TEXT_SCORE_BLUE, 2);
        self.draw_centered_text(left_center, table_top + 20, "GREATEST", TEXT_SCORE_BLUE, 2);
        self.draw_centered_text(right_center, table_top, "ALL TIME", TEXT_SCORE_BLUE, 2);
        self.draw_centered_text(right_center, table_top + 20, "GREATEST", TEXT_SCORE_BLUE, 2);

        let left_x = rect.x + 40;
        let right_x = rect.center_x() + 24;
        let row_count = todays.entries().len().max(all_time.entries().len());
        for index in 0..row_count {
            self.draw_text(
                left_x,
                rect.y + 58 + index as i32 * 24,
                &arcade_score_row(index + 1, todays.entries().get(index)),
                TEXT_SCORE_BLUE,
                2,
            );
            self.draw_text(
                right_x,
                rect.y + 58 + index as i32 * 24,
                &arcade_score_row(index + 1, all_time.entries().get(index)),
                TEXT_SCORE_BLUE,
                2,
            );
        }
    }

    fn draw_space_backdrop(&mut self, seed: u32, clip: Option<Rect>) {
        let rect = clip.unwrap_or(Rect {
            x: 0,
            y: 0,
            width: self.image_width as i32,
            height: self.image_height as i32,
        });

        if clip.is_none() {
            self.fill_rect(rect, Color::from_rgba(BACKGROUND));
        }

        for index in 0..180u32 {
            let hash = hash32(seed.wrapping_mul(97).wrapping_add(index * 7919));
            let x = rect.x + (hash % rect.width.max(1) as u32) as i32;
            let y = rect.y + ((hash.rotate_left(13)) % rect.height.max(1) as u32) as i32;
            let brightness = 150 + ((hash >> 24) as u8 % 100);
            self.draw_dot(x, y, Color(brightness, brightness, 255, 255), 1);
        }
    }

    fn draw_title_logo(&mut self, center_x: i32, top_y: i32) {
        self.draw_centered_text(center_x, top_y - 64, "DEFENDER", TEXT_DANGER, 7);
        self.draw_centered_text(
            center_x,
            top_y + 6,
            "RED LABEL ARCADE REIMPLEMENTATION",
            TEXT_SECONDARY,
            2,
        );
        self.draw_line(
            150,
            top_y + 40,
            self.image_width as i32 - 150,
            top_y + 40,
            Color::from_rgba([36, 64, 120, 255]),
            2,
        );
    }

    fn draw_arcade_logo(&mut self, center_x: i32, top_y: i32, scale: i32) {
        for depth in (1..=3).rev() {
            self.draw_centered_text(
                center_x + depth * scale,
                top_y + depth * scale,
                "DEFENDER",
                TEXT_DANGER,
                scale,
            );
        }
        self.draw_centered_text(center_x, top_y, "DEFENDER", TEXT_WARNING, scale);
    }

    fn draw_secret_status(
        &mut self,
        xyzzy_active: bool,
        invincible: bool,
        auto_fire: bool,
        x: i32,
        y: i32,
    ) {
        if !xyzzy_active {
            return;
        }

        self.draw_text(x, y, "XYZZY MODE ENABLED", TEXT_WARNING, 2);
        self.draw_text(
            x + 244,
            y,
            if invincible {
                "GOD MODE ON"
            } else {
                "GOD MODE OFF"
            },
            if invincible {
                TEXT_WARNING
            } else {
                TEXT_SECONDARY
            },
            2,
        );
        self.draw_text(
            x + 436,
            y,
            if auto_fire {
                "AUTO FIRE ON"
            } else {
                "AUTO FIRE OFF"
            },
            if auto_fire {
                TEXT_WARNING
            } else {
                TEXT_SECONDARY
            },
            2,
        );
        self.draw_text(x, y + 22, "SMART BOMBS INF", TEXT_PRIMARY, 1);
    }

    fn default_scanner_rect(&self) -> Rect {
        Rect {
            x: 24,
            y: 86,
            width: self.image_width as i32 - 48,
            height: 34,
        }
    }

    fn default_playfield_rect(&self) -> Rect {
        Rect {
            x: 24,
            y: 136,
            width: self.image_width as i32 - 48,
            height: self.image_height as i32 - 236,
        }
    }

    fn draw_panel(&mut self, rect: Rect) {
        self.fill_rect(rect, Color::from_rgba(PANEL_BACKGROUND));
        self.stroke_rect(rect, Color::from_rgba(PANEL_BORDER), 2);
    }

    fn draw_text(&mut self, x: i32, y: i32, text: &str, color: [u8; 4], scale: i32) {
        let font = arcade_font();
        let glyph_color = Color::from_rgba(color);
        let scale = scale.max(1);
        let mut pen_x = x;
        for ch in text.chars() {
            let glyph = font.glyph_for_char(ch);
            self.draw_scaled_glyph(glyph.image(), pen_x, y, glyph_color, scale);
            pen_x += glyph.advance() * scale;
        }
    }

    fn draw_centered_text(
        &mut self,
        center_x: i32,
        y: i32,
        text: &str,
        color: [u8; 4],
        scale: i32,
    ) {
        let width = arcade_font().text_width(text, scale);
        self.draw_text(center_x - width / 2, y, text, color, scale);
    }

    fn draw_scaled_glyph(
        &mut self,
        glyph: &RenderedImage,
        origin_x: i32,
        origin_y: i32,
        color: Color,
        scale: i32,
    ) {
        if glyph.width == 0 || glyph.height == 0 {
            return;
        }

        for src_y in 0..glyph.height {
            for src_x in 0..glyph.width {
                let index = ((src_y * glyph.width + src_x) * 4) as usize;
                let alpha = glyph.pixels[index + 3];
                if alpha == 0 {
                    continue;
                }
                for sy in 0..scale {
                    for sx in 0..scale {
                        self.render_target.blend_pixel(
                            origin_x + src_x as i32 * scale + sx,
                            origin_y + src_y as i32 * scale + sy,
                            Color(color.0, color.1, color.2, alpha),
                        );
                    }
                }
            }
        }
    }

    fn draw_scaled_image_centered(
        &mut self,
        image: &RenderedImage,
        center_x: i32,
        center_y: i32,
        target_height: i32,
    ) {
        if image.width == 0 || image.height == 0 {
            return;
        }

        let target_height = target_height.max(1);
        let target_width = (((image.width as i32) * target_height) / image.height as i32).max(1);
        let origin_x = center_x - target_width / 2;
        let origin_y = center_y - target_height / 2;

        for dy in 0..target_height {
            let src_y = ((dy as u32) * image.height / target_height as u32).min(image.height - 1);
            for dx in 0..target_width {
                let src_x = ((dx as u32) * image.width / target_width as u32).min(image.width - 1);
                let index = ((src_y * image.width + src_x) * 4) as usize;
                let alpha = image.pixels[index + 3];
                if alpha == 0 {
                    continue;
                }
                self.render_target.blend_pixel(
                    origin_x + dx,
                    origin_y + dy,
                    Color(
                        image.pixels[index],
                        image.pixels[index + 1],
                        image.pixels[index + 2],
                        alpha,
                    ),
                );
            }
        }
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) {
        for y in rect.y.max(0)..(rect.y + rect.height).min(self.image_height as i32) {
            for x in rect.x.max(0)..(rect.x + rect.width).min(self.image_width as i32) {
                self.render_target.put_pixel(x, y, color);
            }
        }
    }

    fn stroke_rect(&mut self, rect: Rect, color: Color, thickness: i32) {
        self.draw_line(
            rect.x,
            rect.y,
            rect.x + rect.width,
            rect.y,
            color,
            thickness,
        );
        self.draw_line(
            rect.x,
            rect.y,
            rect.x,
            rect.y + rect.height,
            color,
            thickness,
        );
        self.draw_line(
            rect.x + rect.width,
            rect.y,
            rect.x + rect.width,
            rect.y + rect.height,
            color,
            thickness,
        );
        self.draw_line(
            rect.x,
            rect.y + rect.height,
            rect.x + rect.width,
            rect.y + rect.height,
            color,
            thickness,
        );
    }

    fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color, thickness: i32) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let (mut x, mut y) = (x0, y0);

        loop {
            self.stamp(x, y, color, thickness);
            if x == x1 && y == y1 {
                break;
            }
            let doubled = err * 2;
            if doubled >= dy {
                err += dy;
                x += sx;
            }
            if doubled <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn draw_dot(&mut self, cx: i32, cy: i32, color: Color, radius: i32) {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    self.render_target.put_pixel(cx + dx, cy + dy, color);
                }
            }
        }
    }

    fn stamp(&mut self, x: i32, y: i32, color: Color, thickness: i32) {
        let radius = thickness.saturating_sub(1);
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                self.render_target.put_pixel(x + dx, y + dy, color);
            }
        }
    }
}

impl RenderedImage {
    pub fn new_blank(width: u32, height: u32, color: [u8; 4]) -> Self {
        let mut image = Self {
            width,
            height,
            pixels: vec![0; width as usize * height as usize * 4],
        };
        image.clear(Color::from_rgba(color));
        image
    }

    fn resize(&mut self, width: u32, height: u32, color: [u8; 4]) {
        self.width = width;
        self.height = height;
        self.pixels.resize(width as usize * height as usize * 4, 0);
        self.clear(Color::from_rgba(color));
    }

    fn clear(&mut self, color: Color) {
        for pixel in self.pixels.chunks_exact_mut(4) {
            pixel[0] = color.0;
            pixel[1] = color.1;
            pixel[2] = color.2;
            pixel[3] = color.3;
        }
    }

    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 {
            return;
        }

        let x = usize::try_from(x).ok();
        let y = usize::try_from(y).ok();
        let (Some(x), Some(y)) = (x, y) else {
            return;
        };
        if x >= self.width as usize || y >= self.height as usize {
            return;
        }

        let index = (y * self.width as usize + x) * 4;
        self.pixels[index] = color.0;
        self.pixels[index + 1] = color.1;
        self.pixels[index + 2] = color.2;
        self.pixels[index + 3] = color.3;
    }

    fn blend_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 {
            return;
        }

        let x = usize::try_from(x).ok();
        let y = usize::try_from(y).ok();
        let (Some(x), Some(y)) = (x, y) else {
            return;
        };
        if x >= self.width as usize || y >= self.height as usize {
            return;
        }

        let index = (y * self.width as usize + x) * 4;
        if color.3 == 255 {
            self.pixels[index] = color.0;
            self.pixels[index + 1] = color.1;
            self.pixels[index + 2] = color.2;
            self.pixels[index + 3] = 255;
            return;
        }

        let alpha = u16::from(color.3);
        let inverse = 255_u16.saturating_sub(alpha);
        self.pixels[index] =
            ((u16::from(color.0) * alpha + u16::from(self.pixels[index]) * inverse) / 255) as u8;
        self.pixels[index + 1] = ((u16::from(color.1) * alpha
            + u16::from(self.pixels[index + 1]) * inverse)
            / 255) as u8;
        self.pixels[index + 2] = ((u16::from(color.2) * alpha
            + u16::from(self.pixels[index + 2]) * inverse)
            / 255) as u8;
        self.pixels[index + 3] = 255;
    }
}

impl Color {
    fn from_rgba([r, g, b, a]: [u8; 4]) -> Self {
        Self(r, g, b, a)
    }
}

impl Rect {
    fn inset(self, amount: i32) -> Self {
        Self {
            x: self.x + amount,
            y: self.y + amount,
            width: (self.width - amount * 2).max(1),
            height: (self.height - amount * 2).max(1),
        }
    }

    fn center_x(self) -> i32 {
        self.x + self.width / 2
    }

    fn center_y(self) -> i32 {
        self.y + self.height / 2
    }
}

fn raster_size(geometry: TerminalGeometry) -> (u32, u32) {
    let source_width = if geometry.pixel_width > 0 {
        geometry.pixel_width as u32
    } else {
        u32::from(geometry.cols.max(40)) * 16
    };
    let source_height = if geometry.pixel_height > 0 {
        geometry.pixel_height as u32
    } else {
        u32::from(geometry.rows.max(18)) * 32
    };

    scale_to_fit(
        source_width,
        source_height,
        MAX_RENDER_WIDTH,
        MAX_RENDER_HEIGHT,
    )
}

fn scale_to_fit(width: u32, height: u32, max_width: u32, max_height: u32) -> (u32, u32) {
    if width == 0 || height == 0 {
        return (LOGICAL_WIDTH, LOGICAL_HEIGHT);
    }

    let scale = (max_width as f32 / width as f32)
        .min(max_height as f32 / height as f32)
        .min(1.0);

    let scaled_width = ((width as f32 * scale).round() as u32).max(LOGICAL_WIDTH);
    let scaled_height = ((height as f32 * scale).round() as u32).max(LOGICAL_HEIGHT);
    (scaled_width, scaled_height)
}

fn hash32(mut value: u32) -> u32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^ (value >> 16)
}

fn arcade_score_row(rank: usize, entry: Option<&HighScoreEntry>) -> String {
    match entry {
        Some(entry) => format!("{rank} {:<3} {:>5}", entry.initials, entry.score),
        None => format!("{rank} --- -----"),
    }
}

fn color_for_enemy_name(name: &str) -> [u8; 4] {
    match name {
        "LANDER" => LANDER_COLOR,
        "MUTANT" => MUTANT_COLOR,
        "BAITER" => BAITER_COLOR,
        "BOMBER" => BOMBER_COLOR,
        "POD" => POD_COLOR,
        "SWARMER" => SWARMER_COLOR,
        _ => TEXT_PRIMARY,
    }
}

fn scanner_color(kind: EntityKind) -> [u8; 4] {
    match kind {
        EntityKind::PlayerShip => PLAYER_COLOR,
        EntityKind::PlayerShot => PLAYER_SHOT_COLOR,
        EntityKind::EnemyShot => ENEMY_SHOT_COLOR,
        EntityKind::Lander => LANDER_COLOR,
        EntityKind::Mutant => MUTANT_COLOR,
        EntityKind::Baiter => BAITER_COLOR,
        EntityKind::Bomber => BOMBER_COLOR,
        EntityKind::Pod => POD_COLOR,
        EntityKind::Swarmer => SWARMER_COLOR,
        EntityKind::Mine => MINE_COLOR,
        EntityKind::Human => HUMAN_COLOR,
    }
}

fn sprite_draw_height(kind: EntityKind, scale: i32) -> i32 {
    match kind {
        EntityKind::PlayerShip => scale * 2,
        EntityKind::PlayerShot => (scale / 2).max(3),
        EntityKind::EnemyShot => scale.max(5),
        EntityKind::Human => (scale * 2).max(8),
        EntityKind::Lander | EntityKind::Mutant | EntityKind::Bomber | EntityKind::Pod => {
            (scale * 2).max(10)
        }
        EntityKind::Baiter | EntityKind::Swarmer | EntityKind::Mine => (scale * 3 / 2).max(8),
    }
}

#[cfg(test)]
mod tests {
    use super::{Renderer, Screen, scale_to_fit};
    use crate::{
        high_scores::HighScoreTable, render::InitialsEntryView, terminal::TerminalGeometry,
    };

    fn sample_pixel(image: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
        let index = ((y * width + x) * 4) as usize;
        [
            image[index],
            image[index + 1],
            image[index + 2],
            image[index + 3],
        ]
    }

    #[test]
    fn renderer_honours_logical_minimum_size() {
        let renderer = Renderer::new(TerminalGeometry {
            cols: 0,
            rows: 0,
            pixel_width: 0,
            pixel_height: 0,
        });

        assert_eq!(renderer.image_width(), super::LOGICAL_WIDTH);
        assert_eq!(renderer.image_height(), super::LOGICAL_HEIGHT);
    }

    #[test]
    fn gameplay_frame_contains_non_background_pixels() {
        let mut renderer = Renderer::with_size(960, 720);
        let world = crate::game::World::bootstrap();

        let image = renderer.render(Screen::Playing {
            world: &world,
            xyzzy_active: false,
            invincible: false,
            auto_fire: false,
        });

        assert_ne!(
            sample_pixel(
                &image.pixels,
                image.width,
                image.width / 2,
                image.height / 2
            ),
            super::BACKGROUND
        );
    }

    #[test]
    fn initials_screen_renders_score_tables() {
        let mut renderer = Renderer::with_size(960, 720);
        let world = crate::game::World::bootstrap();
        let todays = HighScoreTable::default();
        let all_time = HighScoreTable::default();
        let view = InitialsEntryView {
            high_score: 10_000,
            todays_high_scores: &todays,
            all_time_high_scores: &all_time,
            entry_score: 9_999,
            entry_rank: 1,
            initials: "ABC",
            xyzzy_active: true,
            invincible: true,
            auto_fire: true,
        };

        let image = renderer.render(Screen::InitialsEntry {
            world: &world,
            view: &view,
        });

        assert_ne!(
            sample_pixel(&image.pixels, image.width, image.width / 2, 140),
            super::BACKGROUND
        );
    }

    #[test]
    fn game_over_screen_renders_arcade_strip_and_center_overlay() {
        let mut renderer = Renderer::with_size(960, 720);
        let world = crate::game::World::bootstrap();

        let image = renderer.render(Screen::GameOver {
            world: &world,
            high_score: 10_000,
            xyzzy_active: false,
            invincible: false,
            auto_fire: false,
        });

        assert_ne!(
            sample_pixel(&image.pixels, image.width, 20, 82),
            super::BACKGROUND
        );
        assert_ne!(
            sample_pixel(
                &image.pixels,
                image.width,
                image.width / 2,
                image.height / 2
            ),
            super::BACKGROUND
        );
    }

    #[test]
    fn scale_to_fit_respects_bounds() {
        assert_eq!(scale_to_fit(3_840, 2_160, 1_280, 960), (1_280, 720));
        assert_eq!(
            scale_to_fit(0, 0, 1_280, 960),
            (super::LOGICAL_WIDTH, super::LOGICAL_HEIGHT)
        );
    }
}
