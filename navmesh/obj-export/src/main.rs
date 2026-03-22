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
        .expect("Couldn't convert usize to u32");
    index
        .checked_add(1)
        .expect("Indices must be < 4_294_967_295")
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let asset_cache = AssetCache::new(args.path, &["adr", "cdt", "gcnk"])
        .await
        .expect("Failed to build asset cache");

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

        for triangle_indices in asset.chunk.indices.chunks(3) {
            let triangle = [
                vertex_index(&chunk_to_global_indices, &triangle_indices, 0),
                vertex_index(&chunk_to_global_indices, &triangle_indices, 1),
                vertex_index(&chunk_to_global_indices, &triangle_indices, 2),
            ];
            triangles.push(triangle);
        }
    }

    let mut obj = String::new();

    for vertex in vertices {
        writeln!(&mut obj, "v {} {} {}", vertex[0], vertex[1], vertex[2])
            .expect("Failed to write vertex");
    }

    for triangle in triangles {
        writeln!(
            &mut obj,
            "f {} {} {}",
            triangle[0], triangle[1], triangle[2]
        )
        .expect("Failed to write triangle");
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
