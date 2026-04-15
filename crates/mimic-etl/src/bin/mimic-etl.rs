//! CLI tool for MIMIC ETL operations.
//!
//! Requires the `cli` feature flag:
//! ```bash
//! cargo run -p mimic-etl --features cli --bin mimic-etl -- --help
//! ```

use clap::{Parser, Subcommand};
use mimic_etl::{DatasetConfig, MimicCsvReader, MimicVersion};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mimic-etl", about = "MIMIC clinical database ETL tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert MIMIC CSV files to Parquet/Arrow format
    Convert {
        /// Input directory containing MIMIC CSV files
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory for converted files
        #[arg(short, long)]
        output: PathBuf,

        /// Output format
        #[arg(short, long, default_value = "parquet")]
        format: String,

        /// MIMIC version (3 or 4)
        #[arg(short = 'V', long, default_value = "4")]
        version: u8,

        /// Tables to convert (comma-separated, or 'all')
        #[arg(short, long, default_value = "all")]
        tables: String,
    },

    /// Print the ClinicalEvent Arrow schema
    Schema,

    /// Show info about MIMIC CSV files in a directory
    Info {
        /// Input directory containing MIMIC CSV files
        #[arg(short, long)]
        input: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert {
            input,
            output,
            format,
            version,
            tables,
        } => cmd_convert(&input, &output, &format, version, &tables),
        Commands::Schema => cmd_schema(),
        Commands::Info { input } => cmd_info(&input),
    }
}

fn cmd_convert(input: &PathBuf, output: &PathBuf, format: &str, version: u8, tables: &str) {
    let mimic_version = match version {
        3 => MimicVersion::MimicIII,
        4 => MimicVersion::MimicIV,
        _ => {
            eprintln!("Error: unsupported MIMIC version {version}. Use 3 or 4.");
            std::process::exit(1);
        }
    };

    let config = match mimic_version {
        MimicVersion::MimicIII => DatasetConfig::mimic_iii(input.display().to_string()),
        MimicVersion::MimicIV => DatasetConfig {
            root_path: input.display().to_string(),
            ..DatasetConfig::default()
        },
    };

    let table_list: Vec<String> = if tables == "all" {
        config.tables.clone()
    } else {
        tables.split(',').map(|s| s.trim().to_string()).collect()
    };

    let reader = MimicCsvReader::new(config);

    // Ensure output directory exists
    if let Err(e) = std::fs::create_dir_all(output) {
        eprintln!("Error creating output directory: {e}");
        std::process::exit(1);
    }

    println!(
        "Converting {mimic_version} tables from {} to {format}",
        input.display()
    );

    for table in &table_list {
        // Try .csv first, fall back to .csv.gz
        let csv_path = input.join(format!("{table}.csv"));
        let file_path = if csv_path.exists() {
            csv_path
        } else {
            let gz_path = input.join(format!("{table}.csv.gz"));
            if gz_path.exists() {
                gz_path
            } else {
                eprintln!("  Skipping {table}: file not found (.csv or .csv.gz)");
                continue;
            }
        };

        print!("  {table}...");
        match reader.read_table(table, &file_path) {
            Ok(batches) => {
                let out_path = match format {
                    "parquet" => output.join(format!("{}.parquet", table.to_lowercase())),
                    "arrow" | "ipc" => output.join(format!("{}.arrow", table.to_lowercase())),
                    _ => {
                        eprintln!(" unsupported format: {format}");
                        continue;
                    }
                };

                let result = match format {
                    "parquet" => mimic_etl::to_parquet(&batches, &out_path),
                    "arrow" | "ipc" => mimic_etl::to_arrow_ipc(&batches, &out_path),
                    _ => unreachable!(),
                };

                match result {
                    Ok(()) => {
                        let total_rows: usize = batches.iter().map(|b| b.num_rows()).sum();
                        println!(" {total_rows} rows → {}", out_path.display());
                    }
                    Err(e) => eprintln!(" write error: {e}"),
                }
            }
            Err(e) => eprintln!(" parse error: {e}"),
        }
    }

    println!("Done.");
}

fn cmd_schema() {
    let schema = mimic_etl::types::clinical_event_schema();
    println!("ClinicalEvent Arrow Schema:");
    println!("{schema}");
}

fn cmd_info(input: &PathBuf) {
    use std::io::{BufRead, BufReader};

    println!("MIMIC CSV files in {}:", input.display());
    println!();

    let csv_files = match std::fs::read_dir(input) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Error reading directory: {e}");
            std::process::exit(1);
        }
    };

    let mut found = false;
    for entry in csv_files.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "csv") {
            found = true;
            let name = path.file_stem().unwrap_or_default().to_string_lossy();

            // Count rows using buffered reader (handles multi-GB files)
            match std::fs::File::open(&path) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    let line_count = reader.lines().count();
                    let row_count = if line_count > 0 { line_count - 1 } else { 0 };
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    println!(
                        "  {name:<30} {row_count:>10} rows  ({:.1} MB)",
                        size as f64 / 1_048_576.0
                    );
                }
                Err(e) => println!("  {name:<30} error: {e}"),
            }
        }
    }

    if !found {
        println!("  No CSV files found.");
    }
}
