use asset_serialize::{
    adr::{Adr, AdrData, CollisionData},
    cdt::Cdt,
    gcnk::Gcnk,
};
use clap::Parser;
use flate2::{Compression, write::GzEncoder};
use glam::{EulerRot, Quat, Vec3A};
use kiddo::{SquaredEuclidean, float::kdtree::KdTree};
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    fs::File,
    num::NonZero,
    path::PathBuf,
};
use tokio::fs;

use crate::{
    asset_cache::AssetCache,
    bvh::{BvhFile, BvhInstance, BvhTemplate, generate_bvh},
};

mod asset_cache;
mod bvh;

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

    /// Path to output file. If unspecified, prints to stdout
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Path to bounding volume hierarchy output file (gzipped). If unspecified, does not construct BVH
    #[arg(long, value_name = "BVH_FILE")]
    bvh: Option<PathBuf>,
}

type VertexKdTree = KdTree<f32, usize, 3, 1024, u32>;

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
    global_bvhs: &mut Vec<BvhInstance>,
    bvh_cache: &mut HashMap<String, BvhTemplate>,
    vertex_kd_tree: &mut VertexKdTree,
    obj: &mut String,
) {
    writeln!(obj, "g terrain").expect("Failed to write terrain group");

    for (chunk_index, (asset_name, asset)) in chunks.iter().enumerate() {
        let chunk_to_global_indices = add_vertices(
            asset.chunk.vertices.iter().map(|vertex| vertex.pos),
            global_vertices,
            vertex_kd_tree,
            merge_radius,
        );
        let mut chunk_triangles: Vec<[u16; 3]> = Vec::new();

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

            for triangle in batch_indices.chunks(3).map(|triangle_indices| {
                [
                    vertex_index(batch_vertices, triangle_indices, 0),
                    vertex_index(batch_vertices, triangle_indices, 1),
                    vertex_index(batch_vertices, triangle_indices, 2),
                ]
            }) {
                writeln!(obj, "f {} {} {}", triangle[0], triangle[1], triangle[2])
                    .expect("Failed to write terrain triangle");
            }

            chunk_triangles.extend(
                batch_indices
                    .chunks(3)
                    .map(|triangle| [triangle[0], triangle[1], triangle[2]]),
            );
        }

        let chunk_vertices: Vec<[f32; 3]> = asset
            .chunk
            .vertices
            .iter()
            .map(|vertex| vertex.pos)
            .collect();
        let bvh = generate_bvh(&chunk_vertices, &chunk_triangles);
        let bvh_name = format!("{asset_name}_{chunk_index}");
        bvh_cache.insert(
            bvh_name.clone(),
            BvhTemplate {
                bvh,
                vertices: chunk_vertices,
                triangles: chunk_triangles,
            },
        );
        global_bvhs.push(BvhInstance {
            name: bvh_name,
            pos: [0.0; 3],
            rot: [0.0; 3],
        });
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
    global_bvhs: &mut Vec<BvhInstance>,
    bvh_cache: &mut HashMap<String, BvhTemplate>,
    vertex_kd_tree: &mut VertexKdTree,
    obj: &mut String,
) {
    let mut written_objects = HashSet::new();

    for (_, asset) in chunks.iter() {
        for tile in asset.chunk.tiles.iter() {
            for runtime_obj in tile.runtime_objects.iter() {
                if !written_objects.insert(runtime_obj.terrain_object_identifier.clone()) {
                    continue;
                }

                let Some(cdt_names) = adr_to_cdts.get(&runtime_obj.adr_name) else {
                    continue;
                };

                let cdts = cdt_names.iter().filter_map(|cdt_name| {
                    let cdt = cdts.get(cdt_name).map(|cdt| (cdt_name, cdt));
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
                .expect("Failed to write object group");

                for (cdt_name, cdt) in cdts {
                    for entry in cdt.entries.iter() {
                        let local_to_global_indices = add_vertices(
                            entry.vertices.iter().map(|vertex| {
                                let rotation = Quat::from_euler(
                                    EulerRot::YXZ,
                                    runtime_obj.rot[0],
                                    runtime_obj.rot[1],
                                    runtime_obj.rot[2],
                                );
                                let vertex = rotation
                                    * (Vec3A::new(vertex[0], vertex[1], vertex[2])
                                        * runtime_obj.scale);
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
                                .expect("Failed to write object triangle");
                        }

                        bvh_cache.entry(cdt_name.clone()).or_insert_with(|| {
                            let triangles = entry.triangles.clone();
                            let bvh = generate_bvh(&entry.vertices, &triangles);

                            BvhTemplate {
                                bvh,
                                vertices: entry.vertices.clone(),
                                triangles,
                            }
                        });
                        global_bvhs.push(BvhInstance {
                            name: cdt_name.clone(),
                            pos: [runtime_obj.pos[0], runtime_obj.pos[1], runtime_obj.pos[2]],
                            rot: [runtime_obj.rot[0], runtime_obj.rot[1], runtime_obj.rot[2]],
                        });
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

    let re =
        Regex::new(&format!("^{}_-?\\d+_-?\\d+\\.gcnk$", args.zone)).expect("Invalid chunk regex");
    let gcnk_names = asset_cache
        .filter(&args.zone, |asset_name| re.is_match(asset_name))
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
    let mut global_bvhs: Vec<BvhInstance> = Vec::new();
    let mut bvh_cache: HashMap<String, BvhTemplate> = HashMap::new();
    let mut vertex_kd_tree: VertexKdTree = KdTree::new();

    let mut index_obj = String::new();
    build_terrain(
        &chunks,
        args.merge_radius,
        &mut global_vertices,
        &mut global_bvhs,
        &mut bvh_cache,
        &mut vertex_kd_tree,
        &mut index_obj,
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
        &mut global_bvhs,
        &mut bvh_cache,
        &mut vertex_kd_tree,
        &mut index_obj,
    )
    .await;

    let mut combined_obj = String::new();
    for vertex in global_vertices.iter() {
        writeln!(combined_obj, "v {} {} {}", vertex[0], vertex[1], vertex[2])
            .expect("Failed to write vertex");
    }
    write!(combined_obj, "{}", index_obj).expect("Failed to write all indices");

    match args.output {
        Some(out_path) => {
            fs::write(out_path, combined_obj)
                .await
                .expect("Unable to write to output file");
        }
        None => print!("{}", combined_obj),
    }

    if let Some(bvh_path) = args.bvh {
        let bvh = BvhFile {
            bvhs: bvh_cache,
            references: global_bvhs,
        };
        let file = File::create(bvh_path).expect("Unable to create BVH output file");
        let serialized_bvh: Vec<u8> = pot::to_vec(&bvh).expect("Unable to serialize BVH");
        let mut encoder = GzEncoder::new(file, Compression::best());
        std::io::Write::write_all(&mut encoder, &serialized_bvh)
            .expect("Unable to write to BVH output file");
        encoder
            .finish()
            .expect("Unable to write end of stream to BVH output file");
    }
}
