//! Renders Defender scenes into RGBA frames for Kitty graphics output and README media.

use crate::{
    attract::AttractFrame,
    attract_rom::attract_rom,
    branding::arcade_branding,
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
const TEXT_ATTRACT_PURPLE: [u8; 4] = [182, 96, 255, 255];
const TEXT_ATTRACT_MAGENTA: [u8; 4] = [220, 116, 255, 255];
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
const LOGO_PAGE_X_OFFSET: i32 = -96;
const ATTRACT_TABLE_XS: [i32; 6] = [0x0900, 0x1100, 0x1980, 0x0960, 0x1160, 0x19E0];
const ATTRACT_TABLE_YS: [i32; 6] = [0x6000, 0x6000, 0x6200, 0x9800, 0x9800, 0x9A00];
const ATTRACT_LEGEND_LABEL_OFFSET_Y: i32 = 44;
const ATTRACT_LEGEND_SCORE_OFFSET_Y: i32 = 70;
// These entries follow the ROM `TEXTAB` / `ENMYTB` order for the attract
// instruction page. Their actual screen anchors are derived at render time from
// the ROM object coordinates so the label/score text stays aligned under the
// cabinet positions instead of hand-placed screen columns.
const ATTRACT_SCORE_CARD: [AttractLegendEntry; 6] = [
    AttractLegendEntry::new(EntityKind::Lander, "LANDER", 150),
    AttractLegendEntry::new(EntityKind::Mutant, "MUTANT", 150),
    AttractLegendEntry::new(EntityKind::Baiter, "BAITER", 200),
    AttractLegendEntry::new(EntityKind::Bomber, "BOMBER", 250),
    AttractLegendEntry::new(EntityKind::Pod, "POD", 1000),
    AttractLegendEntry::new(EntityKind::Swarmer, "SWARMER", 150),
];

pub enum Screen<'a> {
    Logo {
        palette_phase: usize,
        elapsed_ms: u64,
        trace_points: usize,
        show_title_text: bool,
        show_full_defender: bool,
        defender_appear_tick: Option<u8>,
        show_copyright: bool,
    },
    Attract {
        frame: &'a AttractFrame,
        palette_phase: usize,
    },
    HighScores {
        todays: &'a HighScoreTable,
        all_time: &'a HighScoreTable,
        palette_phase: usize,
        elapsed_ms: u64,
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

#[derive(Clone, Copy)]
struct AttractLegendEntry {
    kind: EntityKind,
    label: &'static str,
    score: u32,
}

#[derive(Clone, Copy)]
struct AttractPalette {
    williams: [u8; 4],
    title_text: [u8; 4],
    defender_face: [u8; 4],
    defender_shadow: [u8; 4],
    hall_text: [u8; 4],
    scanner_text: [u8; 4],
    scanner_border: [u8; 4],
}

#[derive(Clone, Copy)]
struct LogoScreenState {
    palette_phase: usize,
    elapsed_ms: u64,
    trace_points: usize,
    show_title_text: bool,
    show_full_defender: bool,
    defender_appear_tick: Option<u8>,
    show_copyright: bool,
}

impl AttractLegendEntry {
    const fn new(kind: EntityKind, label: &'static str, score: u32) -> Self {
        Self { kind, label, score }
    }
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
            Screen::Logo {
                palette_phase,
                elapsed_ms,
                trace_points,
                show_title_text,
                show_full_defender,
                defender_appear_tick,
                show_copyright,
            } => self.render_logo_screen(LogoScreenState {
                palette_phase,
                elapsed_ms,
                trace_points,
                show_title_text,
                show_full_defender,
                defender_appear_tick,
                show_copyright,
            }),
            Screen::Attract {
                frame,
                palette_phase,
            } => self.render_attract_screen(frame, palette_phase),
            Screen::HighScores {
                todays,
                all_time,
                palette_phase,
                elapsed_ms,
            } => self.render_high_scores_screen(todays, all_time, palette_phase, elapsed_ms),
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

    fn render_logo_screen(&mut self, state: LogoScreenState) {
        self.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width: self.image_width as i32,
                height: self.image_height as i32,
            },
            Color::from_rgba(BACKGROUND),
        );
        let branding = arcade_branding();
        let rom = attract_rom();
        let palette = attract_palette(state.palette_phase, state.elapsed_ms);
        let max_width = (self.image_width as i32 - 64).max(1);
        let max_height = (self.image_height as i32 - 24).max(1);
        let scale = (max_width as f32 / 320.0).min(max_height as f32 / 256.0);
        let page_width = (320.0 * scale).round() as i32;
        let page_height = (256.0 * scale).round() as i32;
        let page_x = self.image_width as i32 / 2 - page_width / 2 + LOGO_PAGE_X_OFFSET;
        let page_y = self.image_height as i32 / 2 - page_height / 2;
        let page_scale = page_height as f32 / 256.0;
        let page_x_at = |x: f32| page_x + (x * page_width as f32 / 320.0).round() as i32;
        let page_y_at = |y: f32| page_y + (y * page_height as f32 / 256.0).round() as i32;
        let page_rect = Rect {
            x: page_x,
            y: page_y,
            width: page_width,
            height: page_height,
        };

        if state.trace_points >= rom.williams_points().len()
            && state.show_title_text
            && state.show_full_defender
        {
            // Once the ROM-driven trace and object-appearance phases have
            // completed, show the exact embedded red-label page composition,
            // optionally masking the copyright line during the ROM's short
            // post-appear hold.
            self.recolor_attract_page(page_rect, palette);
            if !state.show_copyright {
                self.clear_copyright_line(page_rect, page_scale);
            }
            return;
        }

        for &(x, y) in rom.williams_points().iter().take(state.trace_points) {
            self.draw_scaled_logo_trace_pixel(page_rect, x, y, Color::from_rgba(palette.williams));
        }

        if state.show_title_text {
            self.draw_centered_text(
                page_x_at(187.0),
                page_y_at(88.0),
                "ELECTRONICS INC.",
                palette.title_text,
                page_scale.round().max(1.0) as i32,
            );
            self.draw_centered_text(
                page_x_at(190.0),
                page_y_at(108.0),
                "PRESENTS",
                palette.title_text,
                page_scale.round().max(1.0) as i32,
            );
        }

        if let Some(defender_appear_tick) = state.defender_appear_tick
            && !state.show_full_defender
        {
            self.draw_defender_appear_phase(
                page_rect,
                rom.defender_chunks(),
                defender_appear_tick,
                palette,
            );
        }

        if state.show_copyright {
            self.draw_scaled_image_centered(
                branding.copyright_line(),
                page_x_at(197.0),
                page_y_at(211.0),
                (branding.copyright_line().height as f32 * page_scale)
                    .round()
                    .max(1.0) as i32,
            );
        }
    }

    fn render_attract_screen(&mut self, frame: &AttractFrame, palette_phase: usize) {
        let world = &frame.world;
        let palette = attract_palette(palette_phase, 0);
        // `LEDRET` rebuilds the cabinet playfield via `SCINIT`, `BORDER`,
        // `SCPROC`, and `TEXTP`, so the attract demo uses the same broad
        // scanner/playfield composition as the cabinet instead of the normal
        // gameplay HUD.
        let strip_y = 118;
        self.draw_attract_scanner(
            frame,
            Rect {
                x: self.image_width as i32 / 2 - 168,
                y: 18,
                width: 336,
                height: 70,
            },
        );
        self.draw_line(
            0,
            strip_y,
            self.image_width as i32,
            strip_y,
            Color::from_rgba(palette.scanner_border),
            2,
        );
        self.draw_centered_text(
            self.image_width as i32 / 2,
            128,
            "SCANNER",
            palette.scanner_text,
            3,
        );
        let playfield = Rect {
            x: 0,
            y: strip_y + 6,
            width: self.image_width as i32,
            height: self.image_height as i32 - strip_y - 6,
        };
        self.fill_rect(playfield, Color::from_rgba(VIEWPORT_BACKGROUND));
        self.draw_space_backdrop(world.tick().wrapping_add(17), Some(playfield));
        if !world.planet_destroyed() {
            let mut previous = None;
            for screen_x in 0..world.width() {
                let x = playfield.x
                    + ((screen_x as f32 + 0.5) * playfield.width as f32 / world.width() as f32)
                        .round() as i32;
                let y = playfield.y
                    + ((world.terrain_row_at_screen_x(screen_x) as f32 + 0.5)
                        * playfield.height as f32
                        / world.height() as f32)
                        .round() as i32;
                self.draw_line(
                    x,
                    y,
                    x,
                    playfield.y + playfield.height - 1,
                    Color::from_rgba(TERRAIN_AMBER_FILL),
                    1,
                );
                if let Some((prev_x, prev_y)) = previous {
                    self.draw_line(
                        prev_x,
                        prev_y,
                        x,
                        y,
                        Color::from_rgba(TERRAIN_AMBER_LINE),
                        2,
                    );
                }
                previous = Some((x, y));
            }
        }
        self.draw_attract_demo_objects(frame, playfield);
        if let Some(bonus_text) = frame.bonus_text {
            self.draw_attract_bonus_text(bonus_text, playfield, TEXT_WARNING);
        }
        self.draw_attract_legend_entries(playfield, frame.revealed_score_entries);
    }

    fn render_high_scores_screen(
        &mut self,
        todays: &HighScoreTable,
        all_time: &HighScoreTable,
        palette_phase: usize,
        elapsed_ms: u64,
    ) {
        let palette = attract_palette(palette_phase, elapsed_ms);
        self.fill_rect(
            Rect {
                x: 0,
                y: 0,
                width: self.image_width as i32,
                height: self.image_height as i32,
            },
            Color::from_rgba(BACKGROUND),
        );
        // `HALDIS` writes the whole `DEFENDER` logo through `CWRIT` with
        // yellow face / red shadow on this page. The cabinet video keeps that
        // ROM logo stable while the headings and tables cycle in purple.
        self.draw_defender_logo(self.image_width as i32 / 2, 72, 78, None);
        self.draw_centered_text(
            self.image_width as i32 / 2,
            142,
            "HALL OF FAME",
            palette.hall_text,
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
            palette.hall_text,
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
            TEXT_SCORE_BLUE,
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

    fn draw_attract_scanner(&mut self, frame: &AttractFrame, rect: Rect) {
        let world = &frame.world;
        self.stroke_rect(rect, Color::from_rgba(SCANNER_BORDER), 2);
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

        for object in &frame.objects {
            let x = project_attract_x(inner, object.x16);
            let y = project_attract_y(inner, object.y16);
            let radius = if object.kind == EntityKind::PlayerShip {
                3
            } else {
                2
            };
            self.draw_dot(x, y, Color::from_rgba(scanner_color(object.kind)), radius);
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
        if entity.kind == EntityKind::PlayerShot && entity.velocity.dx != 0 {
            self.draw_player_shot_beam(entity, cx, cy, scale);
            return;
        }

        let sprites = arcade_sprites();
        let image = sprites.sprite_for_entity(entity, tick, facing);
        self.draw_scaled_image_centered(
            image.as_ref(),
            cx,
            cy,
            sprite_draw_height(entity.kind, scale),
        );
    }

    fn draw_player_shot_beam(&mut self, entity: &Entity, cx: i32, cy: i32, scale: i32) {
        // Defender's live gameplay laser reads as a short horizontal beam on the
        // cabinet, not a chunky projectile sprite. The attract sequence still
        // uses the ROM-driven laser path separately, so this beam only applies
        // to moving in-game shots.
        let direction = entity.velocity.dx.signum();
        let half_length = scale.max(4);
        let core_half_length = (half_length - 2).max(2);
        let tail_x = cx - direction * half_length;
        let core_tail_x = cx - direction * core_half_length;

        self.draw_line(
            tail_x,
            cy,
            cx,
            cy,
            Color::from_rgba([255, 232, 104, 255]),
            2,
        );
        self.draw_line(
            core_tail_x,
            cy,
            cx,
            cy,
            Color::from_rgba(TEXT_ARCADE_WHITE),
            1,
        );
        self.stamp(cx + direction, cy, Color::from_rgba(TEXT_ARCADE_WHITE), 1);
    }

    fn draw_attract_demo_objects(&mut self, frame: &AttractFrame, rect: Rect) {
        let mut player_shot_points = Vec::new();
        for object in &frame.objects {
            let cx = project_attract_x(rect, object.x16);
            let cy = project_attract_y(rect, object.y16);
            if object.kind == EntityKind::PlayerShot {
                player_shot_points.push((cx, cy));
                continue;
            }
            let entity = Entity::with_state(object.kind, 0, 0, 0, 0, object.state);
            self.draw_entity(&entity, object.facing, 0, cx, cy, 24);
        }

        if !player_shot_points.is_empty() {
            for window in player_shot_points.windows(2) {
                let (x0, y0) = window[0];
                let (x1, y1) = window[1];
                self.draw_line(x0, y0, x1, y1, Color::from_rgba([255, 232, 104, 255]), 2);
                self.draw_line(x0, y0, x1, y1, Color::from_rgba(TEXT_ARCADE_WHITE), 1);
            }
            let (head_x, head_y) = *player_shot_points.last().expect("shot head");
            self.stamp(head_x, head_y, Color::from_rgba(TEXT_ARCADE_WHITE), 1);
        }
    }

    fn draw_attract_bonus_text(
        &mut self,
        bonus_text: crate::attract::AttractBonusText,
        rect: Rect,
        color: [u8; 4],
    ) {
        let x = project_attract_x(rect, bonus_text.x16);
        let y = project_attract_y(rect, bonus_text.y16);
        self.draw_centered_text(x, y, bonus_text.text, color, 2);
    }

    fn draw_score_tables(
        &mut self,
        rect: Rect,
        todays: &HighScoreTable,
        all_time: &HighScoreTable,
        color: [u8; 4],
    ) {
        let left_center = rect.x + rect.width / 4;
        let right_center = rect.x + rect.width * 3 / 4;
        let table_top = rect.y + 6;
        self.draw_centered_text(left_center, table_top, "TODAYS", color, 2);
        self.draw_centered_text(left_center, table_top + 20, "GREATEST", color, 2);
        self.draw_centered_text(right_center, table_top, "ALL TIME", color, 2);
        self.draw_centered_text(right_center, table_top + 20, "GREATEST", color, 2);

        let left_x = rect.x + 40;
        let right_x = rect.center_x() + 24;
        let row_count = todays.entries().len().max(all_time.entries().len());
        for index in 0..row_count {
            self.draw_text(
                left_x,
                rect.y + 58 + index as i32 * 24,
                &arcade_score_row(index + 1, todays.entries().get(index)),
                color,
                2,
            );
            self.draw_text(
                right_x,
                rect.y + 58 + index as i32 * 24,
                &arcade_score_row(index + 1, all_time.entries().get(index)),
                color,
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

    fn draw_defender_logo(
        &mut self,
        center_x: i32,
        center_y: i32,
        target_height: i32,
        palette: Option<AttractPalette>,
    ) {
        match palette {
            Some(palette) => self.draw_scaled_image_centered_remapped(
                arcade_branding().defender_logo(),
                center_x,
                center_y,
                target_height,
                &[
                    ([112, 255, 52, 255], palette.defender_face),
                    ([255, 48, 48, 255], palette.defender_shadow),
                ],
            ),
            None => self.draw_scaled_image_centered(
                arcade_branding().defender_logo(),
                center_x,
                center_y,
                target_height,
            ),
        }
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

    fn draw_attract_legend_entries(&mut self, rect: Rect, revealed_score_entries: usize) {
        for (index, entry) in ATTRACT_SCORE_CARD
            .into_iter()
            .take(revealed_score_entries)
            .enumerate()
        {
            let color = attract_legend_color(entry.kind);
            let x = project_attract_x(rect, ATTRACT_TABLE_XS[index]);
            let label_y =
                project_attract_y(rect, ATTRACT_TABLE_YS[index]) + ATTRACT_LEGEND_LABEL_OFFSET_Y;
            self.draw_centered_text(x, label_y, entry.label, color, 2);
            self.draw_centered_text(
                x,
                label_y + (ATTRACT_LEGEND_SCORE_OFFSET_Y - ATTRACT_LEGEND_LABEL_OFFSET_Y),
                &entry.score.to_string(),
                color,
                2,
            );
        }
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

    fn draw_scaled_image_centered_remapped(
        &mut self,
        image: &RenderedImage,
        center_x: i32,
        center_y: i32,
        target_height: i32,
        remap: &[([u8; 4], [u8; 4])],
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
                let mut rgba = [
                    image.pixels[index],
                    image.pixels[index + 1],
                    image.pixels[index + 2],
                    alpha,
                ];
                if let Some((_, replacement)) = remap.iter().find(|(source, _)| {
                    source[0] == rgba[0]
                        && source[1] == rgba[1]
                        && source[2] == rgba[2]
                        && source[3] == rgba[3]
                }) {
                    rgba = *replacement;
                    rgba[3] = alpha;
                }
                self.render_target.blend_pixel(
                    origin_x + dx,
                    origin_y + dy,
                    Color(rgba[0], rgba[1], rgba[2], rgba[3]),
                );
            }
        }
    }

    fn recolor_attract_page(&mut self, rect: Rect, palette: AttractPalette) {
        let page = arcade_branding().logo_page();
        self.draw_scaled_image_centered_remapped(
            page,
            rect.center_x(),
            rect.center_y(),
            rect.height,
            &[
                ([237, 42, 47, 255], palette.williams),
                ([241, 182, 57, 255], palette.title_text),
                ([112, 255, 52, 255], palette.defender_face),
                ([255, 48, 48, 255], palette.defender_shadow),
            ],
        );
    }

    fn draw_scaled_logo_trace_pixel(
        &mut self,
        page_rect: Rect,
        native_x: u16,
        native_y: u16,
        color: Color,
    ) {
        let display_x0 = i32::from(native_x) * 5 / 4;
        let display_x1 = (i32::from(native_x) + 1) * 5 / 4;
        let display_y0 = i32::from(native_y);
        let display_y1 = i32::from(native_y) + 1;

        let x0 = page_rect.x + display_x0 * page_rect.width / 320;
        let x1 = page_rect.x + display_x1 * page_rect.width / 320;
        let y0 = page_rect.y + display_y0 * page_rect.height / 256;
        let y1 = page_rect.y + display_y1 * page_rect.height / 256;

        self.fill_rect(
            Rect {
                x: x0,
                y: y0,
                width: (x1 - x0).max(1),
                height: (y1 - y0).max(1),
            },
            color,
        );
    }

    fn draw_defender_appear_phase(
        &mut self,
        page_rect: Rect,
        chunks: &[RenderedImage],
        defender_appear_tick: u8,
        palette: AttractPalette,
    ) {
        if chunks.is_empty() {
            return;
        }

        const LOGO_LEFT_BYTE: i32 = 0x30;
        const LOGO_TOP_SCANLINE: i32 = 0x90;
        const CHUNK_WIDTH_BYTES: i32 = 4;
        const CHUNK_HEIGHT_ROW_PAIRS: i32 = 12;
        const CHUNK_CENTER_X_BYTES: i32 = 2;
        const CHUNK_CENTER_Y_SCANLINES: i32 = 12;

        // `EXPU4` starts drawing at size $2E and counts down to 1 before the
        // attract task swaps in the full-width `CWRIT` logo.
        let size = (0x2E_i32 - i32::from(defender_appear_tick)).max(1);
        let row_pair_step = size * 2;

        for (chunk_index, chunk) in chunks.iter().enumerate() {
            let logo_left_byte = LOGO_LEFT_BYTE + chunk_index as i32 * CHUNK_WIDTH_BYTES;
            let start_x_byte = logo_left_byte + CHUNK_CENTER_X_BYTES - CHUNK_CENTER_X_BYTES * size;
            let start_y =
                LOGO_TOP_SCANLINE + CHUNK_CENTER_Y_SCANLINES - CHUNK_CENTER_Y_SCANLINES * size;

            for byte_column in 0..CHUNK_WIDTH_BYTES {
                let target_x_byte = start_x_byte + byte_column * size;
                let source_x = (byte_column * 2) as usize;

                for row_pair in 0..CHUNK_HEIGHT_ROW_PAIRS {
                    let target_y = start_y + row_pair * row_pair_step;
                    let source_y = (row_pair * 2) as usize;
                    self.draw_native_logo_word(
                        page_rect,
                        chunk,
                        (source_x, source_y),
                        (target_x_byte, target_y),
                        palette,
                    );
                }
            }

            // `DONE` erases the center word after each expanded-object write.
            self.clear_native_logo_word(
                page_rect,
                logo_left_byte + CHUNK_CENTER_X_BYTES,
                LOGO_TOP_SCANLINE + CHUNK_CENTER_Y_SCANLINES,
            );
        }
    }

    fn draw_native_logo_word(
        &mut self,
        page_rect: Rect,
        chunk: &RenderedImage,
        source_origin: (usize, usize),
        native_origin: (i32, i32),
        palette: AttractPalette,
    ) {
        for dy in 0..2usize {
            for dx in 0..2usize {
                let pixel_x = source_origin.0 + dx;
                let pixel_y = source_origin.1 + dy;
                if pixel_x >= chunk.width as usize || pixel_y >= chunk.height as usize {
                    continue;
                }

                let index = (pixel_y * chunk.width as usize + pixel_x) * 4;
                let alpha = chunk.pixels[index + 3];
                if alpha == 0 {
                    continue;
                }

                self.fill_raw_page_rect(
                    page_rect,
                    native_origin.0 * 2 + dx as i32,
                    native_origin.1 + dy as i32,
                    1,
                    1,
                    Color::from_rgba(remap_defender_logo_color(
                        [
                            chunk.pixels[index],
                            chunk.pixels[index + 1],
                            chunk.pixels[index + 2],
                            alpha,
                        ],
                        palette,
                    )),
                );
            }
        }
    }

    fn clear_native_logo_word(&mut self, page_rect: Rect, byte_x: i32, scanline_y: i32) {
        self.fill_raw_page_rect(
            page_rect,
            byte_x * 2,
            scanline_y,
            2,
            2,
            Color::from_rgba(BACKGROUND),
        );
    }

    fn clear_copyright_line(&mut self, page_rect: Rect, page_scale: f32) {
        let copyright = arcade_branding().copyright_line();
        let target_height = (copyright.height as f32 * page_scale).round().max(1.0) as i32;
        let target_width =
            (((copyright.width as i32) * target_height) / copyright.height as i32).max(1);
        let center_x = page_rect.x + (197.0 * page_rect.width as f32 / 320.0).round() as i32;
        let center_y = page_rect.y + (211.0 * page_rect.height as f32 / 256.0).round() as i32;
        self.fill_rect(
            Rect {
                x: center_x - target_width / 2 - 4,
                y: center_y - target_height / 2 - 4,
                width: target_width + 8,
                height: target_height + 8,
            },
            Color::from_rgba(BACKGROUND),
        );
    }

    fn fill_raw_page_rect(
        &mut self,
        page_rect: Rect,
        raw_x: i32,
        native_y: i32,
        raw_width: i32,
        native_height: i32,
        color: Color,
    ) {
        let x0 = page_rect.x + (raw_x * 5 / 4) * page_rect.width / 320;
        let x1 = page_rect.x + ((raw_x + raw_width) * 5 / 4) * page_rect.width / 320;
        let y0 = page_rect.y + native_y * page_rect.height / 256;
        let y1 = page_rect.y + (native_y + native_height) * page_rect.height / 256;
        self.fill_rect(
            Rect {
                x: x0,
                y: y0,
                width: (x1 - x0).max(1),
                height: (y1 - y0).max(1),
            },
            color,
        );
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

const ATTRACT_COLOR_CYCLE_MS: u64 = 120;

fn attract_palette(phase: usize, elapsed_ms: u64) -> AttractPalette {
    // The red-label attract path keeps the `COLR` and `TIECOL` color tasks
    // alive while the Williams page and hall-of-fame page are on screen. The
    // hardware color values are packed pseudo-color bytes, so the visible RGB
    // tones here stay video-matched, but the phase advancement follows the
    // ROM's continuously running attract color tasks instead of a static
    // per-page palette.
    match (phase + (elapsed_ms / ATTRACT_COLOR_CYCLE_MS) as usize) % 4 {
        0 => AttractPalette {
            williams: [255, 72, 96, 255],
            title_text: [248, 192, 64, 255],
            defender_face: [112, 255, 52, 255],
            defender_shadow: [255, 48, 48, 255],
            hall_text: TEXT_ATTRACT_PURPLE,
            scanner_text: TEXT_ATTRACT_PURPLE,
            scanner_border: [67, 114, 198, 255],
        },
        1 => AttractPalette {
            williams: [255, 92, 112, 255],
            title_text: [248, 208, 96, 255],
            defender_face: [144, 255, 80, 255],
            defender_shadow: [255, 72, 56, 255],
            hall_text: TEXT_ATTRACT_MAGENTA,
            scanner_text: TEXT_ATTRACT_MAGENTA,
            scanner_border: [82, 132, 220, 255],
        },
        2 => AttractPalette {
            williams: [255, 64, 88, 255],
            title_text: [236, 184, 56, 255],
            defender_face: [96, 240, 48, 255],
            defender_shadow: [236, 40, 72, 255],
            hall_text: [164, 88, 244, 255],
            scanner_text: [164, 88, 244, 255],
            scanner_border: [60, 106, 188, 255],
        },
        _ => AttractPalette {
            williams: [255, 80, 104, 255],
            title_text: [255, 216, 120, 255],
            defender_face: [176, 255, 96, 255],
            defender_shadow: [255, 108, 64, 255],
            hall_text: [206, 108, 255, 255],
            scanner_text: [206, 108, 255, 255],
            scanner_border: [94, 146, 230, 255],
        },
    }
}

fn remap_defender_logo_color(source: [u8; 4], palette: AttractPalette) -> [u8; 4] {
    if source == [112, 255, 52, 255] {
        palette.defender_face
    } else if source == [255, 48, 48, 255] {
        palette.defender_shadow
    } else {
        source
    }
}

fn native_attract_x(x16: i32) -> i32 {
    // The attract/instruction page object X coordinates in `amode1.src`
    // (`XSHIP`, `XMAN`, and the `XS` legend table) line up with the cabinet
    // capture when decoded on a 320-wide attract canvas as 11.5-style fixed
    // point rather than the coarser playfield projection used elsewhere.
    ((x16 + 0x10) >> 5).clamp(0, 319)
}

fn native_attract_y(y16: i32) -> i32 {
    ((y16 + 0x80) >> 8).clamp(0, 255)
}

fn project_attract_x(rect: Rect, x16: i32) -> i32 {
    rect.x + native_attract_x(x16) * rect.width / 320
}

fn project_attract_y(rect: Rect, y16: i32) -> i32 {
    rect.y + native_attract_y(y16) * rect.height / 256
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

fn attract_legend_color(kind: EntityKind) -> [u8; 4] {
    match kind {
        EntityKind::Lander => [248, 232, 132, 255],
        EntityKind::Mutant => [102, 232, 255, 255],
        EntityKind::Baiter => [182, 120, 255, 255],
        EntityKind::Bomber => [108, 255, 120, 255],
        EntityKind::Pod => [255, 176, 96, 255],
        EntityKind::Swarmer => [232, 96, 255, 255],
        _ => TEXT_ATTRACT_PURPLE,
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
        game::{Entity, EntityKind, Status, World},
        high_scores::HighScoreTable,
        render::InitialsEntryView,
        terminal::TerminalGeometry,
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
        let world = World::bootstrap();

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
    fn gameplay_player_shot_renders_as_a_horizontal_beam() {
        let mut renderer = Renderer::with_size(960, 720);
        let world = World::with_entities(
            64,
            18,
            Status {
                score: 0,
                lives: 3,
                wave: 1,
            },
            vec![Entity::new(EntityKind::PlayerShot, 32, 6, 2, 0)],
        );

        let image = renderer.render(Screen::Playing {
            world: &world,
            xyzzy_active: false,
            invincible: false,
            auto_fire: false,
        });
        let y = 303;

        assert_ne!(
            sample_pixel(&image.pixels, image.width, 477, y),
            super::BACKGROUND
        );
        assert_ne!(
            sample_pixel(&image.pixels, image.width, 486, y),
            super::BACKGROUND
        );
        assert_eq!(
            sample_pixel(&image.pixels, image.width, 481, y - 8),
            super::VIEWPORT_BACKGROUND
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

    #[test]
    fn logo_screen_palette_cycles_over_elapsed_time() {
        let mut renderer = Renderer::with_size(960, 720);

        let frame_a = renderer
            .render(Screen::Logo {
                palette_phase: 0,
                elapsed_ms: 0,
                trace_points: crate::attract_rom::WILLIAMS_TRACE_POINT_COUNT,
                show_title_text: true,
                show_full_defender: true,
                defender_appear_tick: None,
                show_copyright: true,
            })
            .clone();
        let frame_b = renderer
            .render(Screen::Logo {
                palette_phase: 0,
                elapsed_ms: 240,
                trace_points: crate::attract_rom::WILLIAMS_TRACE_POINT_COUNT,
                show_title_text: true,
                show_full_defender: true,
                defender_appear_tick: None,
                show_copyright: true,
            })
            .clone();

        assert_ne!(frame_a.pixels, frame_b.pixels);
    }

    #[test]
    fn hall_of_fame_palette_cycles_over_elapsed_time() {
        let mut renderer = Renderer::with_size(960, 720);
        let todays = HighScoreTable::default();
        let all_time = HighScoreTable::default();

        let frame_a = renderer
            .render(Screen::HighScores {
                todays: &todays,
                all_time: &all_time,
                palette_phase: 0,
                elapsed_ms: 0,
            })
            .clone();
        let frame_b = renderer
            .render(Screen::HighScores {
                todays: &todays,
                all_time: &all_time,
                palette_phase: 0,
                elapsed_ms: 240,
            })
            .clone();

        assert_ne!(frame_a.pixels, frame_b.pixels);
    }
}
