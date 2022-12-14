use crate::environment::MovementGrid;
use bevy::ecs::component::Component;
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::transform::components::Transform;
use std::f32::consts::PI;
use std::fs::File;
use std::io::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use strum::{AsStaticRef, IntoEnumIterator};
use strum_macros::EnumIter;

pub struct UnitMovement;

impl Plugin for UnitMovement {
    fn build(&self, app: &mut App) {
        app.add_system(calculate_a_star)
            .add_system(move_units)
            .insert_resource(MovementTimer(Timer::new(
                Duration::from_millis(1500),
                TimerMode::Repeating,
            )));
    }
}
// #[derive(Component)]
// pub struct MoveTarget {
//     pub target: Vec3,
// }
const DISTANCE_FACTOR: f32 = 100.0;
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct NodeCoords {
    xy: UVec2,
    h: Option<Heading>,
}
#[derive(Debug, Clone, Copy)]
pub struct PathNode {
    xy: Vec2,
    h: Heading,
}
#[derive(Eq, PartialEq, Hash, Clone, Copy, EnumIter, Debug, Default)]
enum Heading {
    #[default]
    N,
    // NNE,
    NE,
    // NEE,
    E,
    // SEE,
    SE,
    // SSE,
    S,
    // SSW,
    SW,
    // SWW,
    W,
    // NWW,
    NW,
}
#[derive(Component)]
pub struct MoveCommand {
    pub target: Vec2,
    pub path: Vec<PathNode>,
}
#[derive(Component)]
struct Movable {}

#[derive(Resource)]
struct MovementTimer(Timer);
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
struct AStarNode {
    f_score: i32,
    g_score: i32,
    came_from: Option<UVec2>,
}

// pub fn move_units(
//     mut movable_units: Query<(Entity, &mut Transform, &MoveTarget)>,
//     mut commands: Commands,
// ) {
//     for (mut entity, mut transform, movetarget) in movable_units.iter_mut() {
//         if movetarget.target != start {
//             let rotation_xz: f32 = Vec2 {
//                 x: movetarget.target.x - start.x,
//                 y: movetarget.target.z - start.z,
//             }
//             .angle_between(Vec2 { x: 0.0, y: 1.0 });
//             println!("{:?}", rotation_xz);
//             transform.rotation = Quat::from_rotation_y(rotation_xz);
//             start = movetarget.target;
//         }
//         commands.entity(entity).remove::<MoveTarget>();
//     }
// }
fn calculate_a_star(
    mut movables: Query<(Entity, &mut Transform, &mut MoveCommand), Without<Movable>>,
    gridmap: Res<MovementGrid>,
    mut commands: Commands,
) //-> Option<Vec<UVec2>>
{
    for (entity, transform, mut movcmd) in movables.iter_mut() {
        // println!("calculating a*");
        if transform.translation.x == movcmd.target.x && transform.translation.y == movcmd.target.y
        {
            commands.entity(entity).remove::<MoveCommand>();
            continue;
        }
        // println!("Current position {}", transform.translation);
        // println!("Target position {}", movcmd.target);

        let target: UVec2 =
            (movcmd.target / gridmap.settings.cell_size + gridmap.settings.xy_offset).as_uvec2();
        let start: UVec2 = UVec2 {
            x: (transform.translation.x / gridmap.settings.cell_size + gridmap.settings.xy_offset.x)
                as u32,
            y: (transform.translation.z / gridmap.settings.cell_size + gridmap.settings.xy_offset.y)
                as u32,
        };
        println!("start {:?}\ntarget {:?}", start, target);
        let mut movement_grid: Vec<Vec<HashMap<Heading, AStarNode>>> = vec![
            vec![
                Heading::iter()
                    .map(|x| (
                        x.clone(),
                        AStarNode {
                            f_score: -1,
                            g_score: -1,
                            came_from: None
                        }
                    ))
                    .into_iter()
                    .collect();
                gridmap.grid.len()
            ];
            gridmap.grid[0].len()
        ];
        // println!("X_Length: {}, Y_Length: {}, Headings: {}", gridmap.grid.len(), gridmap)
        let mut came_from: HashMap<NodeCoords, NodeCoords> = HashMap::new();
        let mut open_set: HashSet<NodeCoords> = HashSet::from([NodeCoords {
            xy: start,
            h: Some(Heading::N),
        }]);
        movement_grid[start.x as usize][start.y as usize]
            .get_mut(&Heading::N)
            .unwrap()
            .g_score = 0;
        while !open_set.is_empty() {
            let mut current: NodeCoords = NodeCoords {
                xy: UVec2::ZERO,
                h: Some(Heading::N),
            };
            let mut current_cost = 0;
            for open_cell in open_set.clone() {
                let cell: &AStarNode = movement_grid[open_cell.xy.x as usize]
                    [open_cell.xy.y as usize]
                    .get_mut(&open_cell.h.unwrap_or_default())
                    .unwrap();
                // println!("{:?}", open_cell);
                let cell_f_score: i32 = cell.f_score;
                if current_cost == 0 || cell_f_score < current_cost {
                    current = open_cell.clone();
                    current_cost = cell_f_score;
                }
            }
            // let mut f = File::options().append(true).open("example.log").unwrap();

            // f.write_all(format!("{:?}", current).as_bytes());
            let current_node: AStarNode = movement_grid[current.xy.x as usize]
                [current.xy.y as usize]
                .get(&current.h.unwrap_or_default())
                .unwrap()
                .to_owned();

            // println!("current {:?}, target {:?}", current, movcmd.target);
            if current.xy == target {
                let target_vec2: Vec2 = movcmd.target.clone();
                reconstruct_path(&came_from, current, &gridmap)
                    .iter()
                    .enumerate()
                    .for_each(|(i, x)| {
                        if i != 0 {
                            movcmd.path.push(x.clone());
                        }
                    });
                // for node in reconstruct_path(&came_from, current, &gridmap) {
                //     println!("Node {:?}", node);
                //     if node.xy + gridmap.settings.xy_offset.as_uvec2()
                //         != (start - gridmap.settings.xy_offset.as_uvec2())
                //         && node.xy != movcmd.target.as_uvec2()
                //     {
                //         movcmd.path.push(PathNode {
                //             xy: node.xy.as_vec2(),
                //             h: node.h.unwrap_or_default(),
                //         });
                //     } else if node.xy == movcmd.target.as_uvec2() {
                //         println!("Last node");
                //         movcmd.path.push(PathNode {
                //             xy: target_vec2.clone() * gridmap.settings.cell_size
                //                 + gridmap.settings.xy_offset,
                //             h: node.h.unwrap_or_default(),
                //         });
                //     }
                // }
                commands.entity(entity).insert(Movable {});
                return;
            }
            open_set.remove(&current);
            let neighbours = get_neighbours(current.xy, &gridmap);
            // println!("Current: {:?}", current);
            for neighbour in neighbours {
                // println!("{:?}", neighbour);
                let mut neighbour_node: &mut AStarNode = movement_grid[neighbour.xy.x as usize]
                    [neighbour.xy.y as usize]
                    .get_mut(&neighbour.h.unwrap_or_default())
                    .unwrap();
                let tentative_g_score: i32 = current_node.g_score
                    + (inertia_based_inter_cell_movement(current.clone(), neighbour.clone())
                        * DISTANCE_FACTOR) as i32;

                if tentative_g_score < neighbour_node.g_score || neighbour_node.g_score == -1 {
                    neighbour_node.g_score = tentative_g_score;
                    neighbour_node.f_score = tentative_g_score
                        + (heuristical_distance(
                            neighbour,
                            NodeCoords {
                                xy: target,
                                h: None,
                            },
                        ) * DISTANCE_FACTOR) as i32;
                    // println!("neighbour: {:?}", neighbour_node);
                    came_from.insert(neighbour, current);
                    open_set.insert(neighbour);
                }
            }
        }
    }
    return; // None;
}
fn reconstruct_path(
    came_from: &HashMap<NodeCoords, NodeCoords>,
    end: NodeCoords,
    gridmap: &MovementGrid,
) -> Vec<PathNode> {
    let mut total_path: Vec<PathNode> = vec![];

    let mut current: NodeCoords = end;
    current = came_from[&current];
    let endnode: PathNode = PathNode {
        xy: current.xy.as_vec2() - gridmap.settings.xy_offset,
        h: end.h.unwrap_or_default(),
    };

    total_path.push(endnode);
    while came_from.contains_key(&current) {
        current = came_from[&current];
        total_path.push(PathNode {
            xy: (current.xy.as_vec2() - gridmap.settings.xy_offset) * gridmap.settings.cell_size,
            h: current.h.unwrap_or_default(),
        });
        println!(
            "Current xy: {:?}",
            (current.xy.as_vec2() - gridmap.settings.xy_offset) * gridmap.settings.cell_size
        );
        // println!("{:?}", current);
    }
    // println!("{:?}", total_path);
    return total_path;
}
fn calculate_base_inertia(start: &NodeCoords, end: &NodeCoords) -> u32 {
    // println!("Heading in {:?}, Heading out {:?}", heading_in, heading_out);
    let mut penalty: u32 = 0;
    let difference: i32 =
        (start.h.unwrap_or_default() as i32 - end.h.unwrap_or(Heading::N) as i32).abs();
    // let off_course: i32 =
    //     (calculate_heading(&start.xy, &end.xy) as i32 - start.h.unwrap_or_default() as i32).abs();
    let half_headings: i32 = (Heading::iter().len() as f32 / 2.0).ceil() as i32;
    // penalty += (half_headings - (off_course - half_headings).abs()) as u32 * 1;
    penalty += (half_headings - (difference - half_headings).abs()) as u32;
    // penalty *= 20;
    // println!("penalty {}", penalty);
    return penalty;
}
fn inertia_based_inter_cell_movement(from: NodeCoords, to: NodeCoords) -> f32 {
    let inertia: f32 = 20.0;
    let penalty: f32 = calculate_base_inertia(&from, &to) as f32;
    let cost: f32 = from.xy.as_vec2().distance(to.xy.as_vec2()).abs() + (penalty * inertia);
    // println!(
    //     "From: {:?}, to: {:?}, penalty: {:?}, cost: {:?}",
    //     from, to, penalty, cost
    // );
    return cost;
}
fn heuristical_distance(from: NodeCoords, to: NodeCoords) -> f32 {
    return from.xy.as_vec2().distance(to.xy.as_vec2());
}
fn calculate_heading(from: &UVec2, to: &UVec2) -> Heading {
    let diff: IVec2 = to.as_ivec2() - from.as_ivec2();
    let heading: Heading;
    if diff.x == -1 && diff.y == 0 {
        heading = Heading::E
    } else if diff.x == -1 && diff.y == 1 {
        heading = Heading::NE
    } else if diff.x == 0 && diff.y == 1 {
        heading = Heading::N
    } else if diff.x == 1 && diff.y == 1 {
        heading = Heading::NW
    } else if diff.x == 1 && diff.y == 0 {
        heading = Heading::W
    } else if diff.x == 1 && diff.y == -1 {
        heading = Heading::SW
    } else if diff.x == 0 && diff.y == -1 {
        heading = Heading::S
    } else {
        heading = Heading::SE
    }
    return heading;
}
fn check_path_width(current: UVec2, target: UVec2, gridmap: &MovementGrid) -> bool {
    if current.x != target.x && current.y != target.y {
        if gridmap.grid[current.x as usize][target.y as usize] != 0
            && gridmap.grid[target.x as usize][current.y as usize] != 0
        {
            println!("current {} neighbour {}", current, target);
            return false;
        }
    }
    return true;
}
fn get_neighbours(current: UVec2, gridmap: &MovementGrid) -> Vec<NodeCoords> {
    let mut neighbours: Vec<NodeCoords> = Vec::new();
    for x in -1..2 {
        for y in -1..2 {
            let adjacent_cell: IVec2 = IVec2 {
                x: current.x as i32 + x,
                y: current.y as i32 + y,
            };

            if adjacent_cell.x >= 0
                && (adjacent_cell.x as usize) < gridmap.grid.len()
                && adjacent_cell.y >= 0
                && (adjacent_cell.y as usize) < gridmap.grid[0].len()
                && gridmap.grid[adjacent_cell.x as usize][adjacent_cell.y as usize] == 0
                && adjacent_cell.as_uvec2() != current
                && check_path_width(current, adjacent_cell.as_uvec2(), &gridmap)
            {
                neighbours.push(NodeCoords {
                    xy: UVec2 {
                        x: adjacent_cell.x as u32,
                        y: adjacent_cell.y as u32,
                    },
                    h: Some(calculate_heading(&current, &adjacent_cell.as_uvec2())),
                });
            }
        }
    }
    return neighbours;
}

fn move_towards(
    mut transform: &mut Transform,
    speed: f64,
    rotation_speed: f64,
    delta: f64,
    target: &PathNode,
) -> bool {
    let mut target_reached: bool = false;
    let target_scaled: Vec3 = Vec3 {
        x: target.xy.x as f32,
        y: transform.translation.y,
        z: target.xy.y as f32,
    }; // TODO make this dynamic or calculate in the reconstruct_path

    let translation_direction: Vec3 = target_scaled - transform.translation;
    let euler_rotation: (f32, f32, f32) = transform.rotation.to_euler(EulerRot::YXZ);
    let mut directional_euler_fraction: f32 = ((Heading::iter().len() as u32 - target.h as u32)
        as f32
        / (Heading::iter().len() as f32) as f32);
    println!("{}", directional_euler_fraction);
    directional_euler_fraction *= 2.0 * PI;
    println!("{}", directional_euler_fraction);
    directional_euler_fraction = (directional_euler_fraction + 2.0 * PI) % (2.0 * PI);
    if directional_euler_fraction > PI {
        directional_euler_fraction -= 2.0 * PI;
    }

    println!("{}\n", directional_euler_fraction);
    let target_rotation: Vec3 = Vec3 {
        x: 0.0,
        y: directional_euler_fraction,
        z: 0.0,
    };
    // let rotation_direction: f32 = euler_rotation.1
    //     - (std::f64::consts::PI * -2.0 * (target.h as u32 as f64 / Heading::iter().len() as f64)
    //         % (2.0 * std::f64::consts::PI)
    //         + 2.0 * std::f64::consts::PI) as f32
    //         ;
    let rotation_direction: Vec3 = (target_rotation
        - Vec3 {
            x: euler_rotation.1,
            y: euler_rotation.0,
            z: euler_rotation.2,
        })
    .normalize_or_zero()
        * rotation_speed as f32
        * 1.
        * delta as f32;
    if rotation_direction != Vec3::ZERO {
        println!(
            "initial rotation {}, {}, {}",
            euler_rotation.0, euler_rotation.1, euler_rotation.2
        );
        println!("target rotation {} ", target_rotation);
        println!("rotation direction {} \n", rotation_direction);
        transform.rotate(Quat::from_euler(
            EulerRot::YXZ,
            rotation_direction.y,
            rotation_direction.x,
            rotation_direction.z,
        ));
    }
    // transform.rotation = Quat::from_rotation_y(
    //     (std::f64::consts::PI * -2.0 * (target.h as u32 as f64 / Heading::iter().len() as f64)
    //         % (2.0 * std::f64::consts::PI)) as f32,
    // );

    let translation_vector: Vec3 = translation_direction.normalize() * (speed * delta) as f32;

    if translation_vector.length() >= translation_direction.length()
        || translation_direction == Vec3::ZERO
    {
        transform.translation = target_scaled;
        target_reached = true;
    } else {
        transform.translation += translation_vector;
    }
    return target_reached;
}
fn move_units(
    mut movables: Query<(Entity, &mut Transform, &mut MoveCommand), With<Movable>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    // timer.0.tick(time.delta());
    // if timer.0.finished() {
    //     timer.0.set_duration(Duration::from_millis(150));
    let speed: f64 = 1.0;
    let rotation_speed: f64 = 1.0;
    for (entity, mut transform, mut movcmd) in movables.iter_mut() {
        let node: &PathNode;

        match movcmd.path.last() {
            Some(n) => node = n,
            None => {
                commands.entity(entity).remove::<MoveCommand>();
                commands.entity(entity).remove::<Movable>();
                continue;
            }
        }
        if move_towards(
            &mut transform,
            speed,
            rotation_speed,
            time.delta().as_secs_f64(),
            node,
        ) {
            movcmd.path.pop();
        }
    }
    // }
}
