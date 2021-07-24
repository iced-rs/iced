use iced::menu::{self, Menu};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    Custom,
    XKCD,
    Glider,
    SmallExploder,
    Exploder,
    TenCellRow,
    LightweightSpaceship,
    Tumbler,
    GliderGun,
    Acorn,
}

pub static ALL: &[Preset] = &[
    Preset::Custom,
    Preset::XKCD,
    Preset::Glider,
    Preset::SmallExploder,
    Preset::Exploder,
    Preset::TenCellRow,
    Preset::LightweightSpaceship,
    Preset::Tumbler,
    Preset::GliderGun,
    Preset::Acorn,
];

impl Preset {
    pub fn menu() -> Menu<Self> {
        Menu::with_entries(
            ALL.iter()
                .copied()
                .map(|preset| {
                    menu::Entry::item(preset.to_string(), None, preset)
                })
                .collect(),
        )
    }

    pub fn life(self) -> Vec<(isize, isize)> {
        #[rustfmt::skip]
        let cells = match self {
            Preset::Custom => vec![],
            Preset::XKCD => vec![
                "  xxx  ",
                "  x x  ",
                "  x x  ",
                "   x   ",
                "x xxx  ",
                " x x x ",
                "   x  x",
                "  x x  ",
                "  x x  ",
            ],
            Preset::Glider => vec![
                " x ",
                "  x",
                "xxx"
            ],
            Preset::SmallExploder => vec![
                " x ",
                "xxx",
                "x x",
                " x ",
            ],
            Preset::Exploder => vec![
                "x x x",
                "x   x",
                "x   x",
                "x   x",
                "x x x",
            ],
            Preset::TenCellRow => vec![
                "xxxxxxxxxx",
            ],
            Preset::LightweightSpaceship => vec![
                " xxxxx",
                "x    x",
                "     x",
                "x   x ",
            ],
            Preset::Tumbler => vec![
                " xx xx ",
                " xx xx ",
                "  x x  ",
                "x x x x",
                "x x x x",
                "xx   xx",
            ],
            Preset::GliderGun => vec![
                "                        x           ",
                "                      x x           ",
                "            xx      xx            xx",
                "           x   x    xx            xx",
                "xx        x     x   xx              ",
                "xx        x   x xx    x x           ",
                "          x     x       x           ",
                "           x   x                    ",
                "            xx                      ",
            ],
            Preset::Acorn => vec![
                " x     ",
                "   x   ",
                "xx  xxx",
            ],
        };

        let start_row = -(cells.len() as isize / 2);

        cells
            .into_iter()
            .enumerate()
            .flat_map(|(i, cells)| {
                let start_column = -(cells.len() as isize / 2);

                cells
                    .chars()
                    .enumerate()
                    .filter(|(_, c)| !c.is_whitespace())
                    .map(move |(j, _)| {
                        (start_row + i as isize, start_column + j as isize)
                    })
            })
            .collect()
    }
}

impl Default for Preset {
    fn default() -> Preset {
        Preset::XKCD
    }
}

impl std::fmt::Display for Preset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Preset::Custom => "Custom",
                Preset::XKCD => "xkcd #2293",
                Preset::Glider => "Glider",
                Preset::SmallExploder => "Small Exploder",
                Preset::Exploder => "Exploder",
                Preset::TenCellRow => "10 Cell Row",
                Preset::LightweightSpaceship => "Lightweight spaceship",
                Preset::Tumbler => "Tumbler",
                Preset::GliderGun => "Gosper Glider Gun",
                Preset::Acorn => "Acorn",
            }
        )
    }
}
