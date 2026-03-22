use asset_serialize::gcnk::Gcnk;
use clap::Parser;
use kiddo::{SquaredEuclidean, float::kdtree::KdTree};
use std::{fmt::Write, num::NonZero, path::PathBuf};
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

async fn build_terrain(args: &Cli, asset_cache: &AssetCache, obj: &mut String) {
    let asset_names = asset_cache.filter(&args.zone, |asset_name| asset_name.ends_with(".gcnk"));
    let (assets, errors) = asset_cache.deserialize::<Gcnk>(asset_names).await;
    for (asset_name, error) in errors.into_iter() {
        eprintln!(
            "Failed to deserialize {asset_name} when building navmesh for {}: {error:?}",
            args.zone
        );
    }

    if assets.is_empty() {
        eprintln!("No chunks match {}", args.zone);
        return;
    }

    let mut vertices: Vec<[f32; 3]> = Vec::new();
    let mut vertex_kd_tree: VertexKdTree = KdTree::new();
    let mut triangles: Vec<[u32; 3]> = Vec::new();

    // Stitch vertices that are duplicated between chunks
    for (_, asset) in assets.into_iter() {
        let mut chunk_to_global_indices = Vec::with_capacity(asset.chunk.vertices.len());

        for vertex in asset.chunk.vertices.iter() {
            let duplicate = vertex_kd_tree.nearest_n_within::<SquaredEuclidean>(
                &vertex.pos,
                args.merge_radius,
                NonZero::new(1).unwrap(),
                false,
            );
            let global_index = match duplicate[..] {
                [nearest, ..] => nearest.item,
                [] => {
                    let vertex_index = vertices.len();
                    vertex_kd_tree.add(&vertex.pos, vertex_index);
                    vertices.push(vertex.pos);
                    vertex_index
                }
            };

            chunk_to_global_indices.push(global_index);
        }

        for batch in asset.chunk.render_batches.into_iter() {
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
                triangles.push(triangle);
            }
        }
    }

    for vertex in vertices {
        writeln!(obj, "v {} {} {}", vertex[0], vertex[1], vertex[2])
            .expect("Failed to write vertex");
    }

    for triangle in triangles {
        writeln!(obj, "f {} {} {}", triangle[0], triangle[1], triangle[2])
            .expect("Failed to write triangle");
    }
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let asset_cache = AssetCache::new(&args.path, &["adr", "cdt", "gcnk"])
        .await
        .expect("Failed to build asset cache");

    let mut obj = String::new();
    build_terrain(&args, &asset_cache, &mut obj).await;

    match args.output {
        Some(out_path) => {
            fs::write(out_path, obj)
                .await
                .expect("Unable to write to output file");
        }
        None => print!("{}", obj),
    }
}
