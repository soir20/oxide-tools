use std::{fmt::Write, num::NonZero, path::PathBuf};
use asset_serialize::gcnk::{Gcnk, Vertex};
use clap::Parser;
use kiddo::{SquaredEuclidean, float::kdtree::KdTree};
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

fn vertex_index(tree: &VertexKdTree, chunk_vertices: &[Vertex], triangle_vertex_indices: &[u16], index_in_triangle: usize, max_distance: f32) -> u32 {
    let chunk_vertex_index: usize = triangle_vertex_indices[index_in_triangle].try_into().expect("Couldn't convert u16 to usize");
    let nearest = tree.nearest_n_within::<SquaredEuclidean>(&chunk_vertices[chunk_vertex_index].pos, max_distance, NonZero::new(1).unwrap(), false);
    if nearest.len() == 0 {
        panic!("Vertex not found in kd tree but should have already been inserted");
    }

    let index: u32 = nearest[0].item.try_into().expect("Couldn't convert usize to u32");
    index.checked_add(1).expect("Indices must be < 4_294_967_295")
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let asset_cache = AssetCache::new(args.path, &["adr", "cdt", "gcnk"])
        .await
        .expect("Failed to build asset cache");

    let asset_names =
        asset_cache.filter(&args.zone, |asset_name| asset_name.ends_with(".gcnk"));
    let (assets, errors) = asset_cache.deserialize::<Gcnk>(asset_names).await;
    for (asset_name, error) in errors.into_iter() {
        eprintln!("Failed to deserialize {asset_name} when building navmesh for {}: {error:?}", args.zone);
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
        for vertex in asset.chunk.vertices.iter() {
            let duplicate = vertex_kd_tree.nearest_n_within::<SquaredEuclidean>(&vertex.pos, args.merge_radius, NonZero::new(1).unwrap(), false);
            if duplicate.len() == 0 {
                let vertex_index = vertices.len();
                vertex_kd_tree.add(&vertex.pos, vertex_index);
                vertices.push(vertex.pos);
            }
        }

        for triangle_indices in asset.chunk.indices.chunks(3) {
            let triangle = [
                vertex_index(&vertex_kd_tree, &asset.chunk.vertices, triangle_indices, 0, args.merge_radius),
                vertex_index(&vertex_kd_tree, &asset.chunk.vertices, triangle_indices, 1, args.merge_radius),
                vertex_index(&vertex_kd_tree, &asset.chunk.vertices, triangle_indices, 2, args.merge_radius),
            ];
            triangles.push(triangle);
        }
    }

    let mut obj = String::new();

    for vertex in vertices {
        writeln!(&mut obj, "v {} {} {}", vertex[0], vertex[1], vertex[2]).expect("Failed to write vertex");
    }

    for triangle in triangles {
        writeln!(&mut obj, "f {} {} {}", triangle[0], triangle[1], triangle[2]).expect("Failed to write triangle");
    }

    match args.output {
        Some(out_path) => {
            fs::write(out_path, obj).await.expect("Unable to write to output file");
        },
        None => print!("{}", obj),
    }
}
