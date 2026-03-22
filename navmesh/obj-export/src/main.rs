use asset_serialize::{
    adr::{Adr, AdrData, CollisionData},
    cdt::Cdt,
    gcnk::Gcnk,
};
use clap::Parser;
use kiddo::{SquaredEuclidean, float::kdtree::KdTree};
use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    num::NonZero,
    path::PathBuf,
};
use tokio::fs;

use crate::asset_cache::AssetCache;

mod asset_cache;

/// Contains program arguments parsed from the command line
#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    /// Path to assets directory
    #[arg(short, long, value_name = "DIR")]
    path: PathBuf,

    /// Name of the zone asset (without the .gzne extension)
    #[arg(short, long, value_name = "ZONE")]
    zone: String,

    /// Radius in which to merge vertices
    #[arg(short = 'r', long, value_name = "RADIUS")]
    merge_radius: f32,

    /// Path to outout file. If unspecified, prints to stdout
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
}

type VertexKdTree = KdTree<f32, usize, 3, 512, u32>;

fn vertex_index(
    chunk_to_global_indices: &[usize],
    triangle_vertex_indices: &[u16],
    index_in_triangle: usize,
) -> u32 {
    let chunk_vertex_index: usize = triangle_vertex_indices[index_in_triangle].into();
    let index: u32 = chunk_to_global_indices[chunk_vertex_index]
        .try_into()
        .expect("Couldn't convert index to u32");
    index
        .checked_add(1)
        .expect("Indices must be < 4_294_967_295")
}

fn add_vertices(
    local_vertices: impl Iterator<Item = [f32; 3]>,
    global_vertices: &mut Vec<[f32; 3]>,
    vertex_kd_tree: &mut VertexKdTree,
    merge_radius: f32,
) -> Vec<usize> {
    let mut local_to_global_indices = Vec::with_capacity(local_vertices.size_hint().0);

    for vertex in local_vertices {
        // Stitch vertices that are duplicated between chunks
        let duplicate = vertex_kd_tree.nearest_n_within::<SquaredEuclidean>(
            &vertex,
            merge_radius,
            NonZero::new(1).unwrap(),
            false,
        );
        let global_index = match duplicate[..] {
            [nearest, ..] => nearest.item,
            [] => {
                let vertex_index = global_vertices.len();
                vertex_kd_tree.add(&vertex, vertex_index);
                global_vertices.push(vertex);
                vertex_index
            }
        };

        local_to_global_indices.push(global_index);
    }

    local_to_global_indices
}

async fn build_terrain(
    chunks: &[(String, Gcnk)],
    merge_radius: f32,
    global_vertices: &mut Vec<[f32; 3]>,
    vertex_kd_tree: &mut VertexKdTree,
    obj: &mut String,
) {
    writeln!(obj, "g terrain").expect("Failed to write terrain group");

    for (_, asset) in chunks.iter() {
        let chunk_to_global_indices = add_vertices(
            asset.chunk.vertices.iter().map(|vertex| vertex.pos),
            global_vertices,
            vertex_kd_tree,
            merge_radius,
        );

        for batch in asset.chunk.render_batches.iter() {
            let batch_index_start: usize = batch
                .index_offset
                .try_into()
                .expect("Tried to convert batch index start to usize");
            let batch_index_count = batch
                .index_count
                .try_into()
                .expect("Tried to convert batch index count to usize");
            let batch_index_end: usize = batch_index_start
                .checked_add(batch_index_count)
                .expect("Batch index end is out of bounds of usize");
            let batch_indices = &asset.chunk.indices[batch_index_start..batch_index_end];

            let batch_vertex_start: usize = batch
                .vertex_offset
                .try_into()
                .expect("Tried to convert batch vertex start to usize");
            let batch_vertex_count: usize = batch
                .vertex_count
                .try_into()
                .expect("Tried to convert batch vertex end to usize");
            let batch_vertex_end: usize = batch_vertex_start
                .checked_add(batch_vertex_count)
                .expect("Batch vertex end is out of bounds of usize");
            let batch_vertices = &chunk_to_global_indices[batch_vertex_start..batch_vertex_end];

            for triangle_indices in batch_indices.chunks(3) {
                let triangle = [
                    vertex_index(batch_vertices, triangle_indices, 0),
                    vertex_index(batch_vertices, triangle_indices, 1),
                    vertex_index(batch_vertices, triangle_indices, 2),
                ];
                writeln!(obj, "f {} {} {}", triangle[0], triangle[1], triangle[2])
                    .expect("Failed to write terrain triangle");
            }
        }
    }
}

fn list_adrs(chunks: &[(String, Gcnk)]) -> HashSet<&str> {
    chunks
        .iter()
        .flat_map(|(_, asset)| {
            asset.chunk.tiles.iter().flat_map(|tile| {
                tile.runtime_objects
                    .iter()
                    .map(|runtime_obj| runtime_obj.adr_name.as_str())
            })
        })
        .collect()
}

fn map_to_cdt(adrs: &[(String, Adr)]) -> HashMap<String, Vec<String>> {
    adrs.iter()
        .map(|(asset_name, asset)| {
            (
                asset_name.clone(),
                asset
                    .entries
                    .iter()
                    .flat_map(|entry| match &entry.data {
                        AdrData::Collision { entries } => entries
                            .iter()
                            .map(|entry| match &entry.data {
                                CollisionData::AssetName { name } => name.clone(),
                            })
                            .collect(),
                        _ => Vec::new(),
                    })
                    .collect(),
            )
        })
        .collect()
}

fn unique_cdts(adr_to_cdts: &HashMap<String, Vec<String>>) -> HashSet<&str> {
    adr_to_cdts
        .values()
        .flat_map(|cdts| cdts.iter())
        .map(|asset_name| asset_name.as_str())
        .collect()
}

async fn build_objects(
    chunks: &[(String, Gcnk)],
    adr_to_cdts: &HashMap<String, Vec<String>>,
    cdts: &HashMap<String, Cdt>,
    merge_radius: f32,
    global_vertices: &mut Vec<[f32; 3]>,
    vertex_kd_tree: &mut VertexKdTree,
    obj: &mut String,
) {
    for (_, asset) in chunks.iter() {
        for tile in asset.chunk.tiles.iter() {
            for runtime_obj in tile.runtime_objects.iter() {
                let Some(cdt_names) = adr_to_cdts.get(&runtime_obj.adr_name) else {
                    continue;
                };

                let cdts = cdt_names.iter().filter_map(|cdt_name| {
                    let cdt = cdts.get(cdt_name);
                    if cdt.is_none() {
                        eprintln!("Failed to find CDT {}", cdt_name);
                    }
                    cdt
                });

                writeln!(
                    obj,
                    "g {} {}",
                    runtime_obj.adr_name, runtime_obj.terrain_object_identifier
                )
                .expect("Failed to write terrain group");

                for cdt in cdts {
                    for entry in cdt.entries.iter() {
                        let local_to_global_indices = add_vertices(
                            entry.vertices.iter().map(|vertex| {
                                [
                                    vertex[0] + runtime_obj.pos[0],
                                    vertex[1] + runtime_obj.pos[1],
                                    vertex[2] + runtime_obj.pos[2],
                                ]
                            }),
                            global_vertices,
                            vertex_kd_tree,
                            merge_radius,
                        );

                        for triangle_indices in entry.triangles.iter() {
                            let triangle = [
                                vertex_index(&local_to_global_indices, triangle_indices, 0),
                                vertex_index(&local_to_global_indices, triangle_indices, 1),
                                vertex_index(&local_to_global_indices, triangle_indices, 2),
                            ];

                            writeln!(obj, "f {} {} {}", triangle[0], triangle[1], triangle[2])
                                .expect("Failed to write terrain triangle");
                        }
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let asset_cache = AssetCache::new(&args.path, &["adr", "cdt", "gcnk"])
        .await
        .expect("Failed to build asset cache");

    let gcnk_names = asset_cache
        .filter(&args.zone, |asset_name| asset_name.ends_with(".gcnk"))
        .into_iter();
    let (chunks, errors) = asset_cache.deserialize::<Gcnk>(gcnk_names).await;
    for (asset_name, error) in errors.into_iter() {
        eprintln!(
            "Failed to deserialize GCNK {asset_name} when building navmesh for {}: {error:?}",
            args.zone
        );
    }

    if chunks.is_empty() {
        eprintln!("No chunks match {}", args.zone);
        return;
    }

    let mut global_vertices: Vec<[f32; 3]> = Vec::new();
    let mut vertex_kd_tree: VertexKdTree = KdTree::new();

    let mut obj = String::new();
    build_terrain(
        &chunks,
        args.merge_radius,
        &mut global_vertices,
        &mut vertex_kd_tree,
        &mut obj,
    )
    .await;

    let adr_names = list_adrs(&chunks);
    let (adrs, errors) = asset_cache
        .deserialize::<Adr>(adr_names.iter().copied())
        .await;
    for (asset_name, error) in errors.into_iter() {
        eprintln!(
            "Failed to deserialize ADR {asset_name} when building navmesh for {}: {error:?}",
            args.zone
        );
    }

    let adr_to_cdts = map_to_cdt(&adrs);
    let cdt_names = unique_cdts(&adr_to_cdts);
    let (cdts, errors) = asset_cache
        .deserialize::<Cdt>(cdt_names.iter().copied())
        .await;
    for (asset_name, error) in errors.into_iter() {
        eprintln!(
            "Failed to deserialize CDT {asset_name} when building navmesh for {}: {error:?}",
            args.zone
        );
    }

    build_objects(
        &chunks,
        &adr_to_cdts,
        &cdts.into_iter().collect(),
        args.merge_radius,
        &mut global_vertices,
        &mut vertex_kd_tree,
        &mut obj,
    )
    .await;

    for vertex in global_vertices.into_iter() {
        writeln!(obj, "v {} {} {}", vertex[0], vertex[1], vertex[2])
            .expect("Failed to write terrain vertex");
    }

    match args.output {
        Some(out_path) => {
            fs::write(out_path, obj)
                .await
                .expect("Unable to write to output file");
        }
        None => print!("{}", obj),
    }
}
