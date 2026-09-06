use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use fonarch_core::{gather, GatherEvent, GatherOptions};

#[derive(Parser, Debug)]
#[command(
    name = "gather",
    about = "FONarch headless GATHER — copy named livetype fonts to the Desktop"
)]
struct Args {
    /// Print the plan without copying.
    #[arg(long)]
    dry_run: bool,
    /// Parent folder for `FONarch YYYY-MM-DD` (default: Desktop).
    #[arg(long)]
    out: Option<PathBuf>,
    /// Skip hunt and use this livetype directory.
    #[arg(long)]
    livetype: Option<PathBuf>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let opts = GatherOptions {
        dest_parent: args.out,
        dry_run: args.dry_run,
        livetype: args.livetype,
    };

    match gather(opts, |ev| match ev {
        GatherEvent::Status(s) => eprintln!("{s}"),
        GatherEvent::Found { path, via } => {
            eprintln!("Found livetype ({via}): {}", path.display());
        }
        GatherEvent::Plan { total, families } => {
            eprintln!("Catalog {total} fonts / {families} families");
        }
        GatherEvent::Font { name, index, total } => {
            eprintln!("Gathering {name} ({index}/{total})");
        }
        GatherEvent::Warn(s) => eprintln!("warn: {s}"),
        GatherEvent::Zip { name, index, total } => {
            eprintln!("Zipping {name} ({index}/{total})");
        }
        GatherEvent::Done { report } => {
            if report.dry_run {
                eprintln!(
                    "Dry run: {} fonts / {} families would go to {}",
                    report.copied,
                    report.families,
                    report.output.display()
                );
                if report.fallback > 0 {
                    eprintln!("named from OpenType table: {}", report.fallback);
                }
                if report.variable > 0 {
                    eprintln!("variable fonts: {}", report.variable);
                }
            } else {
                eprintln!(
                    "Saved {} fonts / {} families to {}",
                    report.copied,
                    report.families,
                    report.output.display()
                );
                if report.skipped > 0 {
                    eprintln!("skipped {}", report.skipped);
                }
                if report.fallback > 0 {
                    eprintln!("named from OpenType table: {}", report.fallback);
                }
                if report.variable > 0 {
                    eprintln!("variable fonts: {}", report.variable);
                }
            }
        }
    }) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("gather failed: {e}");
            ExitCode::FAILURE
        }
    }
}
