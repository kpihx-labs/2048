use crate::game::layout::{LayoutMode, LayoutOptions, MIN_TILE_HEIGHT, MIN_TILE_WIDTH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliAction {
    Run(LayoutOptions),
    Help,
    Version,
}

pub fn parse_args<I, S>(args: I) -> Result<CliAction, String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into);
    let _program = args.next();
    let mut mode = LayoutMode::Auto;
    let mut tile_width = None;
    let mut tile_height = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(CliAction::Help),
            "-V" | "--version" => return Ok(CliAction::Version),
            "--layout" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--layout requires a value".to_string())?;
                mode = value.parse()?;
            }
            "--tile-width" => {
                tile_width = Some(parse_dimension(
                    "--tile-width",
                    args.next(),
                    MIN_TILE_WIDTH,
                )?);
            }
            "--tile-height" => {
                tile_height = Some(parse_dimension(
                    "--tile-height",
                    args.next(),
                    MIN_TILE_HEIGHT,
                )?);
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    let has_manual_dimensions = tile_width.is_some() || tile_height.is_some();
    if has_manual_dimensions && mode != LayoutMode::Manual {
        return Err("--tile-width and --tile-height require --layout manual".to_string());
    }
    if mode == LayoutMode::Manual && (tile_width.is_none() || tile_height.is_none()) {
        return Err("--layout manual requires both --tile-width and --tile-height".to_string());
    }

    Ok(CliAction::Run(LayoutOptions {
        mode,
        tile_width,
        tile_height,
    }))
}

fn parse_dimension(flag: &str, value: Option<String>, minimum: u16) -> Result<u16, String> {
    let value = value.ok_or_else(|| format!("{flag} requires a value"))?;
    let parsed = value
        .parse::<u16>()
        .map_err(|_| format!("{flag} must be a positive integer"))?;
    if parsed < minimum {
        return Err(format!("{flag} must be at least {minimum}"));
    }
    Ok(parsed)
}

pub fn help() -> &'static str {
    "2048 1.0.0\n\
A responsive terminal implementation of the 2048 puzzle.\n\n\
USAGE:\n\
    2048 [OPTIONS]\n\n\
OPTIONS:\n\
    --layout <MODE>         auto, compact, balanced, large, or manual [default: auto]\n\
    --tile-width <COLUMNS>  Tile width in manual mode (minimum: 5)\n\
    --tile-height <ROWS>    Tile height in manual mode (minimum: 1)\n\
    -h, --help              Print help\n\
    -V, --version           Print version\n\n\
EXAMPLES:\n\
    2048\n\
    2048 --layout compact\n\
    2048 --layout large\n\
    2048 --layout manual --tile-width 18 --tile-height 5"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_auto_layout() {
        assert_eq!(
            parse_args(["2048"]).unwrap(),
            CliAction::Run(LayoutOptions::default())
        );
    }

    #[test]
    fn parses_manual_dimensions() {
        assert_eq!(
            parse_args([
                "2048",
                "--layout",
                "manual",
                "--tile-width",
                "18",
                "--tile-height",
                "5",
            ])
            .unwrap(),
            CliAction::Run(LayoutOptions {
                mode: LayoutMode::Manual,
                tile_width: Some(18),
                tile_height: Some(5),
            })
        );
    }

    #[test]
    fn rejects_partial_manual_dimensions() {
        let error = parse_args(["2048", "--layout", "manual", "--tile-width", "18"]).unwrap_err();
        assert!(error.contains("requires both"));
    }

    #[test]
    fn rejects_manual_dimensions_in_auto_mode() {
        let error = parse_args(["2048", "--tile-width", "18"]).unwrap_err();
        assert!(error.contains("require --layout manual"));
    }
}
