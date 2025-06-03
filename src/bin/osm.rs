#![feature(path_add_extension)]

#[cfg(feature = "osm")]
use std::fs::File;
#[cfg(feature = "osm")]
use std::io::BufWriter;
#[cfg(feature = "osm")]
use std::io::Write;
use std::path::PathBuf;

use anstream::println;
use clap::Parser;
#[cfg(feature = "osm")]
use indoc::indoc;
#[cfg(feature = "osm")]
use owo_colors::OwoColorize;
#[cfg(feature = "osm")]
use tqdm::tqdm;

#[cfg(feature = "osm")]
use search::problems::osm::OSMSpace;

#[cfg(feature = "mem_profile")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
#[cfg(not(feature = "mem_profile"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// Command line arguments
#[derive(Parser, Debug)]
#[clap(long_version = search::build::CLAP_LONG_VERSION)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, env = "LOGS_OSM", default_value = "logs/osm.org")]
    pub output: PathBuf,

    #[arg()]
    pub osm_map: PathBuf,

    #[command(flatten)]
    color: colorchoice_clap::Color,
}

fn split_pbf_file(
    map_file: PathBuf,
    max_nodes: usize,
    max_ways: usize,
    max_relations: usize,
) -> std::io::Result<PathBuf> {
    let db_path = map_file.with_added_extension("duckdb");
    let db = duckdb::Connection::open(&db_path)
        .map_err(|_| std::io::Error::other("Failed creating DB."))?;

    let mut nodes: usize = 0;
    let mut ways: usize = 0;
    let mut relations: usize = 0;
    let mut num_nodes: usize = 0;
    let mut num_ways: usize = 0;
    let mut num_relations: usize = 0;
    {
        // Scan & Write
        use osmio::OSMReader;
        use osmio::obj_types::StringOSMObj;
        use search::problems::osm::OSMState;

        println!("Preparing DB");
        db.execute("begin transaction", [])
            .map_err(|e| std::io::Error::other(format!("Failed to start transaction. {e}")))?;

        println!("Preparing tables");
        // `https://docs.rs/osmio/latest/osmio/trait.Node.html`
        db.execute(
            indoc! {"
                  create table nodes (
                    id bigint not null,

                    lat integer not null,
                    lon integer not null,

                    primary key (id),
                  )
                "},
            [],
        )
        .map_err(|e| std::io::Error::other(format!("Failed creating Node table. {e}")))?;
        let mut nodes_appender = db
            .appender("nodes")
            .map_err(|e| std::io::Error::other(format!("Failed creating Node appender. {e}")))?;

        println!("Preparing Way DB shards");
        // `https://docs.rs/osmio/latest/osmio/trait.Way.html`
        db.execute(
            indoc! {"
                  create table ways (
                    id bigint not null,
                    -- TODO: Add more info

                    -- way_type varchar?,

                    primary key (id),
                  )
                "},
            [],
        )
        .map_err(|e| std::io::Error::other(format!("Failed creating 'ways' table. {e}")))?;
        let mut ways_appender = db
            .appender("ways")
            .map_err(|e| std::io::Error::other(format!("Failed creating ways appender. {e}")))?;

        db.execute(
            indoc! {"
                  create table way_nodes (
                    way_id bigint not null,
                    index smallint not null,
                    node_id bigint not null,


                    primary key (way_id, index),
                  )
                "},
            [],
        )
        .map_err(|e| std::io::Error::other(format!("Failed creating 'way_nodes' table. {e}")))?;
        let mut way_nodes_appender = db.appender("way_nodes").map_err(|e| {
            std::io::Error::other(format!("Failed creating way_nodes appender. {e}"))
        })?;

        println!("Preparing Relation DB shards");
        // `https://docs.rs/osmio/latest/osmio/trait.Relation.html`
        db.execute(
            indoc! {"
                  create table relations (
                    id bigint not null,

                    primary key (id),
                  )
                "},
            [],
        )
        .map_err(|_| std::io::Error::other("Failed creating 'relations' table."))?;
        let mut relations_appender = db.appender("relations").map_err(|e| {
            std::io::Error::other(format!("Failed creating relations appender. {e}"))
        })?;

        db.execute("commit", [])
            .map_err(|e| std::io::Error::other(format!("Failed to commit transaction. {e}")))?;

        for obj in tqdm(
            osmio::read_pbf(&map_file)
                .map_err(|_| std::io::Error::other("Failed reading objects."))?
                .objects(),
        )
        .desc(Some(format!("Reading PBF file {map_file:?}")))
        {
            match obj {
                StringOSMObj::Node(node) => {
                    if num_nodes == 0 {
                        println!("  Inserting Nodes");
                        db.execute("begin transaction", []).map_err(|e| {
                            std::io::Error::other(format!("Failed to start Nodes transaction. {e}"))
                        })?;
                    }
                    if num_nodes % (64 * 1024 * 1024) == 0 {
                        println!("    Commiting Nodes batch...");
                        db.execute("commit", []).map_err(|e| {
                            std::io::Error::other(format!(
                                "Failed to commit Nodes transaction. {e}"
                            ))
                        })?;
                        db.execute("begin transaction", []).map_err(|e| {
                            std::io::Error::other(format!("Failed to start Ways transaction. {e}"))
                        })?;
                    }

                    num_nodes += 1;
                    if num_nodes > max_nodes {
                        if num_ways > max_ways && num_relations > max_relations {
                            break;
                        }
                        continue;
                    }
                    nodes += 1;

                    let node = OSMState::from_osm_node(&node).unwrap();
                    nodes_appender
                        .append_row(duckdb::params![
                            node.id.as_i64(),
                            node.lat.as_i32(),
                            node.lon.as_i32()
                        ])
                        .map_err(|_| std::io::Error::other("Failed inserting Node {node:?}."))?;
                }
                StringOSMObj::Way(way) => {
                    if num_ways == 0 {
                        println!("  Commiting Nodes");
                        db.execute("commit", []).map_err(|e| {
                            std::io::Error::other(format!(
                                "Failed to commit Nodes transaction. {e}"
                            ))
                        })?;
                        println!("  Inserting Ways");
                        db.execute("begin transaction", []).map_err(|e| {
                            std::io::Error::other(format!("Failed to start Ways transaction. {e}"))
                        })?;
                    }
                    if num_ways % (4 * 1024 * 1024) == 0 {
                        println!("    Commiting Ways batch...");
                        db.execute("commit", []).map_err(|e| {
                            std::io::Error::other(format!("Failed to commit Ways transaction. {e}"))
                        })?;
                        db.execute("begin transaction", []).map_err(|e| {
                            std::io::Error::other(format!("Failed to start Ways transaction. {e}"))
                        })?;
                    }
                    num_ways += 1;
                    if num_ways > max_ways {
                        if num_nodes > max_nodes && num_relations > max_relations {
                            break;
                        }
                        continue;
                    }
                    ways += 1;

                    use osmio::OSMObjBase; // `https://docs.rs/osmio/latest/osmio/trait.OSMObjBase.html`
                    use osmio::Way; // `https://docs.rs/osmio/latest/osmio/trait.Way.html`
                    if way.nodes().len() < 2 {
                        continue;
                    }

                    ways_appender
                        .append_row(duckdb::params![way.id()])
                        .map_err(|e| {
                            std::io::Error::other(format!("Failed inserting Way {way:?}. {e}"))
                        })?;

                    // TODO: Verify that all nodes are known?
                    // TODO: Verify that stored nodes are known? This means you can use the way, but only to get to known nodes.
                    for (i, node_id) in way.nodes().iter().enumerate() {
                        way_nodes_appender
                            .append_row(duckdb::params![way.id(), i as u64, *node_id,])
                            .map_err(|e| {
                                std::io::Error::other(format!(
                                    "Failed inserting Way-Node {way:?}. {e}"
                                ))
                            })?;
                    }
                }
                StringOSMObj::Relation(relation) => {
                    if num_relations == 0 {
                        println!("  Committing Ways");
                        db.execute("commit", []).map_err(|e| {
                            std::io::Error::other(format!("Failed to commit Ways transaction. {e}"))
                        })?;
                        println!("  Inserting Relations");
                        db.execute("begin transaction", []).map_err(|e| {
                            std::io::Error::other(format!(
                                "Failed to start Relations transaction. {e}"
                            ))
                        })?;
                    }
                    num_relations += 1;
                    if num_relations > max_relations {
                        if num_nodes > max_nodes && num_ways > max_ways {
                            break;
                        }
                        continue;
                    }
                    relations += 1;

                    use osmio::OSMObjBase;
                    // `https://docs.rs/osmio/latest/osmio/trait.OSMObjBase.html`
                    // `https://docs.rs/osmio/latest/osmio/trait.Relation.html`
                    relations_appender
                        .append_row(duckdb::params![relation.id()])
                        .map_err(|e| {
                            std::io::Error::other(format!(
                                "Failed inserting Relation {relation:?}. {e}"
                            ))
                        })?;

                    // ```rust
                    // for (i, (obj_type, obj_id, rel_name)) in
                    //     relation.members().into_iter().enumerate()
                    // {
                    //     println!("- {i}: {rel_name} {obj_type:?}, {obj_id}");
                    // }
                    // ```
                }
            }
        }

        db.execute("commit", []).map_err(|e| {
            std::io::Error::other(format!("Failed to commit Relations transaction. {e}"))
        })?;
    }

    use thousands::Separable;
    println!("Wrote DB to {}", db_path.display());
    println!(
        "Wrote {}/{} Nodes",
        nodes.separate_with_commas(),
        num_nodes.separate_with_commas(),
    );
    println!(
        "Wrote {}/{} Ways",
        ways.separate_with_commas(),
        num_ways.separate_with_commas(),
    );
    println!(
        "Wrote {}/{} Relations",
        relations.separate_with_commas(),
        num_relations.separate_with_commas(),
    );

    Ok(db_path)
}

#[cfg(not(feature = "osm"))]
fn osm_demo() -> std::io::Result<()> {
    println!("This requires the 'osm' feature.");
    Ok(())
}

#[cfg(feature = "osm")]
fn osm_demo() -> std::io::Result<()> {
    let args = Args::parse();
    args.color.write_global();
    println!("Logging to {:?}", args.output.yellow());

    let file = File::create(&args.output)?;
    let mut out = BufWriter::new(file);

    let max_nodes = 600_000_000;
    let max_ways = 100_000_000;
    let max_relations = 10_000;
    let db = split_pbf_file(args.osm_map.clone(), max_nodes, max_ways, max_relations)
        .map_err(|e| std::io::Error::other(format!("Failed splitting BPF database. {e}")))?;
    println!("DB: {}", db.display());

    writeln!(out, "* Runs")?;
    if let Some(space) = OSMSpace::new(&args.osm_map) {
        println!("Loaded map!");
        println!("{space}");
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
    #[cfg(feature = "coz_profile")]
    coz::thread_init();

    osm_demo()
}
