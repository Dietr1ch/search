//! Implementation of a 2D map.
//!
//! This presents a 2D world using (Latitude, Longitude) pairs.
use std::hash::Hash;
use std::path::PathBuf;

use derive_more::Display;
use thousands::Separable;

// use crate::cost::Cost;
use crate::float_cost::FloatCost;
// use crate::problem::BaseProblem;
// use crate::problem::ObjectiveProblem;
use crate::space::Action;
// use crate::space::ObjectiveHeuristic;
use crate::space::Space;
use crate::space::State;

#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
#[display("{}", _0.degrees())]
pub struct Latitude(osmio::Lat);

#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
#[display("{}", _0.degrees())]
pub struct Longitude(osmio::Lon);

impl std::hash::Hash for Latitude {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.inner().hash(state);
    }
}
impl std::hash::Hash for Longitude {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.inner().hash(state);
    }
}

#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
#[display("OSMId({_0})")]
pub struct OSMId(osmio::ObjId);

#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
#[display("OSMNodeId({_0})")]
pub struct OSMNodeId(OSMId);

#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
#[display("OSMWayId({_0})")]
pub struct OSMWayId(OSMId);

fn lat_long(node: &impl osmio::Node) -> Option<(Latitude, Longitude)> {
    let (lat, long) = node.lat_lon()?;
    let (lat, long) = (Latitude(lat), Longitude(long));
    Some((lat, long))
}

// https://wiki.openstreetmap.org/wiki/Tags
// enum WayTag {
//     Name(String),
//     // https://wiki.openstreetmap.org/wiki/Key:bridge
//     Bridge(String),
//     // https://wiki.openstreetmap.org/wiki/Key:tunnel
//     Tunnel,
//     // https://wiki.openstreetmap.org/wiki/Key:oneway
//     Oneway,
//     // https://wiki.openstreetmap.org/wiki/Key:lanes
//     Lanes(u16),
//     // https://wiki.openstreetmap.org/wiki/Key:ref
//     Ref,
//     // https://wiki.openstreetmap.org/wiki/Key:highway
//     Highway,
//     // Maxspeed,
//     // Service,
//     // Access,
//     // Area,
//     // Landuse,
//     // Width,
//     // EstWidth,
//     // Junction,
//     Other(String, String),
// }

#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, Hash)]
#[display("{id}@({lat},{long})")]
pub struct OSMState {
    // #[serde(rename = "@id")]
    pub id: OSMNodeId,

    // #[serde(rename = "@lat")]
    pub lat: Latitude,
    // #[serde(rename = "@lon")]
    pub long: Longitude,
    // #[serde(rename = "tag", default)]
    // pub tags: Vec<XmlTag>,
}

impl OSMState {
    pub(crate) fn new(id: OSMNodeId, lat_long: (Latitude, Longitude)) -> OSMState {
        let (lat, long) = lat_long;
        OSMState { id, lat, long }
    }
    pub(crate) fn from_osm_node(node: &impl osmio::Node) -> Option<OSMState> {
        if !node.has_lat_lon() {
            return None;
        }

        let lat_long = lat_long(node).unwrap();
        let id = OSMNodeId(OSMId(node.id()));
        Some(Self::new(id, lat_long))
    }
}
impl State for OSMState {}

// Action
#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, PartialOrd)]
#[display("OSMAction({id})")]
pub struct OSMAction {
    /// OSM Id
    pub id: OSMWayId,
    /// Cost as in number of Nodes
    cost: OSMCost,
    // pub uid: u32,  Not set
    // Values reachable from Graph + edge_id,
    // - nodes: Vec<OSMNodeId>,
    // - tags: Vec<XmlTag>,
    // - length: f64,
    // - speed_kph: f64,
    // - walk_travel_time: f64,
    // - bike_travel_time: f64,
    // - drive_travel_time: f64,
}
impl OSMAction {
    pub(crate) fn from_osm_way(way: &impl osmio::Way) -> Option<OSMAction> {
        // https://docs.rs/osmio/latest/osmio/trait.Way.html

        let id = OSMWayId(OSMId(way.id()));
        let cost = FloatCost::<f32>::new(way.nodes().len() as f32);
        Some(Self { id, cost })
    }
}
impl Action for OSMAction {}

// Cost
type OSMCost = FloatCost<f32>;

// Space
use rustc_hash::FxHashMap;
#[derive(Clone, Display)]
#[display("OSMSpace({}, ({}; {}; {}; {}))", path.display(),
nodes.len().separate_with_commas(), ways.len().separate_with_commas(), num_vertices.separate_with_commas(), num_edges.separate_with_commas())]
pub struct OSMSpace {
    pub path: PathBuf,

		// FIXME: Move nodes and ways to disk
    pub nodes: FxHashMap<OSMNodeId, OSMState>,
    pub ways: FxHashMap<OSMWayId, OSMAction>,

    pub actions: FxHashMap<OSMNodeId, Vec<(OSMWayId, OSMCost)>>,

    // Stats
    pub num_vertices: usize, // 43,573,814
    pub num_edges: usize,    // 47,136,848
}

impl OSMSpace {
    pub fn new(map_file: &std::path::Path) -> Option<OSMSpace> {
        use osmio::Node;
        use osmio::Way;
        // use osmio::Relation;
        use osmio::OSMObjBase;
        use osmio::OSMReader;
        use osmio::obj_types::StringOSMObj;

        let mut actions = FxHashMap::<OSMNodeId, Vec<(OSMWayId, OSMCost)>>::default();

        let mut reader = osmio::read_pbf(map_file).ok()?;
        let mut num_actions = 0;
        for obj in reader.objects() {
            match obj {
                StringOSMObj::Node(node) => {
                    // https://docs.rs/osmio/latest/osmio/trait.Node.html
                    if !node.has_lat_lon() {
                        continue;
                    }
                    if node.deleted() {
                        continue;
                    }

                    let _node = OSMState::from_osm_node(&node).unwrap();
                    // nodes.insert(node.id, node);
                }
                StringOSMObj::Way(way) => {
                    // https://docs.rs/osmio/latest/osmio/trait.Way.html
                    if way.num_nodes() < 2 {
                        continue;
                    }
                    // if way.deleted() {
                    // 		continue;
                    // }

                    let action = OSMAction::from_osm_way(&way).unwrap();
                    // ways.insert(action.id, action);

                    if let Some(oneway) = way.tag("oneway") {
                        if oneway == "yes" {
                            num_actions += 1;
                            let last_node = OSMNodeId(OSMId(*way.nodes().last().unwrap()));
                            if let Some(a) = actions.get_mut(&last_node) {
                                a.push((action.id, action.cost));
                            } else {
                                actions.insert(last_node, vec![(action.id, action.cost)]);
                            }
                        }
                    }
                    let first_node = OSMNodeId(OSMId(*way.nodes().first().unwrap()));
                    num_actions += 1;
                    if let Some(a) = actions.get_mut(&first_node) {
                        a.push((action.id, action.cost));
                    } else {
                        actions.insert(first_node, vec![(action.id, action.cost)]);
                    }
                }
                StringOSMObj::Relation(relation) => {
                    // https://docs.rs/osmio/latest/osmio/trait.Relation.html
                    if relation.deleted() {
                        continue;
                    }
                }
            }
        }

        let num_vertices = actions.len();
        Some(OSMSpace {
            path: map_file.to_path_buf(),
            nodes: FxHashMap::default(),
            ways: FxHashMap::default(),
            actions,
            num_vertices,
            num_edges: num_actions,
        })
    }
}

impl Space<OSMState, OSMAction, OSMCost> for OSMSpace {
    fn apply(&self, _s: &OSMState, _a: &OSMAction) -> Option<OSMState> {
        // FIXME
        None
    }
    fn cost(&self, _s: &OSMState, a: &OSMAction) -> OSMCost {
        // FIXME
        println!("{}", a.id);
        OSMCost::infinity()
    }
    fn neighbours(&self, s: &OSMState) -> Vec<(OSMState, OSMAction)> {
        // FIXME
        println!("{}", s.id);
        vec![]
    }
    fn valid(&self, _s: &OSMState) -> bool {
        // FIXME
        false
    }

    fn supports_random_state() -> bool {
        false
    }
    fn random_state<R: rand::Rng>(&self, _r: &mut R) -> Option<OSMState> {
        None
    }
}

impl std::fmt::Debug for OSMSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "OSMSpace({}: ({}; {}; {}; {}))",
            self.path.display(),
            self.nodes.len().separate_with_commas(),
            self.ways.len().separate_with_commas(),
            self.num_vertices.separate_with_commas(),
            self.num_edges.separate_with_commas(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let id = OSMNodeId(OSMId(123));
        let lat = Latitude(osmio::Lat::from_inner(123456));
        let long = Longitude(osmio::Lon::from_inner(123456));
				let lat_long = (lat, long);

        assert_eq!(
            OSMState::new(id, lat_long),
            OSMState {
								id,
                lat,
                long,
            }
        );
    }
}
