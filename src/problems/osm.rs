//! Implementation of a 2D map.
//!
//! This presents a 2D world using (Latitude, Longitude) pairs.
use std::hash::Hash;
use std::path::PathBuf;

use derive_more::Display;
use serde::{Deserialize, Serialize};
use thousands::Separable;

use crate::float_cost::FloatCost;
use crate::space::Action;
use crate::space::Space;
use crate::space::State;

#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
#[display("{}", _0.degrees())]
pub struct Latitude(osmio::Lat);

impl Latitude {
    pub fn as_i32(&self) -> i32 {
        self.0.inner()
    }
}

#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
#[display("{}", _0.degrees())]
pub struct Longitude(osmio::Lon);

impl Longitude {
    pub fn as_i32(&self) -> i32 {
        self.0.inner()
    }
}

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

#[derive(
    Copy, Clone, Debug, Display, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[repr(transparent)]
#[display("OSMId({_0})")]
pub struct OSMId(osmio::ObjId);

impl OSMId {
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

#[derive(
    Copy, Clone, Debug, Display, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[repr(transparent)]
#[display("OSMNodeId({_0})")]
pub struct OSMNodeId(OSMId);

impl OSMNodeId {
    pub fn as_i64(&self) -> i64 {
        self.0.as_i64()
    }
}

#[derive(
    Copy, Clone, Debug, Display, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[repr(transparent)]
#[display("OSMWayId({_0})")]
pub struct OSMWayId(OSMId);

impl OSMWayId {
    pub fn as_i64(&self) -> i64 {
        self.0.as_i64()
    }
}

fn lat_lon(node: &impl osmio::Node) -> Option<(Latitude, Longitude)> {
    let (lat, lon) = node.lat_lon()?;
    let (lat, lon) = (Latitude(lat), Longitude(lon));
    Some((lat, lon))
}

// `https://wiki.openstreetmap.org/wiki/Tags`
// ```rust
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
// ```

#[derive(Copy, Clone, Debug, Display, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[display("{id}@({lat},{lon})")]
pub struct OSMState {
    pub id: OSMNodeId,

    pub lat: Latitude,
    pub lon: Longitude,
}

impl OSMState {
    pub(crate) fn new(id: OSMNodeId, lat_lon: (Latitude, Longitude)) -> OSMState {
        let (lat, lon) = lat_lon;
        OSMState { id, lat, lon }
    }
    pub fn from_osm_node(node: &impl osmio::Node) -> Option<OSMState> {
        if !node.has_lat_lon() {
            return None;
        }

        let lat_lon = lat_lon(node).unwrap();
        let id = OSMNodeId(OSMId(node.id()));
        Some(Self::new(id, lat_lon))
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
}
impl OSMAction {
    pub(crate) fn from_osm_way(way: &impl osmio::Way) -> Option<OSMAction> {
        // `https://docs.rs/osmio/latest/osmio/trait.Way.html`

        let id = OSMWayId(OSMId(way.id()));
        let cost = FloatCost::<f32>::new(way.nodes().len() as f32);
        Some(Self { id, cost })
    }
}
impl Action for OSMAction {}

#[derive(Clone, Debug, Display, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
#[display("OSMWay({id})")]
pub struct OSMWay {
    /// OSM Id
    pub id: OSMWayId,

    pub start: OSMNodeId,
    pub end: OSMNodeId,
}
impl OSMWay {
    pub fn from_osmio_way(way: &impl osmio::Way) -> Option<OSMWay> {
        // `https://docs.rs/osmio/latest/osmio/trait.Way.html`

        let id = OSMWayId(OSMId(way.id()));
        let nodes: Vec<OSMNodeId> = way
            .nodes()
            .iter()
            .map(|obj_id| OSMNodeId(OSMId(*obj_id)))
            .collect();
        if nodes.len() < 2 {
            return None;
        }

        Some(Self {
            id,
            start: *nodes.first().unwrap(),
            end: *nodes.last().unwrap(),
        })
    }
}

#[derive(Clone, Debug, Display, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
#[display("OSMRelation()")]
pub struct OSMRelation {
    pub members: Vec<(osmio::OSMObjectType, osmio::ObjId, String)>,
}
impl OSMRelation {
    pub fn from_osmio_relation(rel: &impl osmio::Relation) -> Option<OSMRelation> {
        // `https://docs.rs/osmio/latest/osmio/trait.Relation.html`

        let members = rel
            .members()
            .map(|(obj_type, id, name)| (obj_type, id, name.to_string()))
            .collect();

        Some(Self { members })
    }
}

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
        use osmio::OSMObjBase;
        use osmio::OSMReader;
        use osmio::Way;
        use osmio::obj_types::StringOSMObj;

        let mut actions = FxHashMap::<OSMNodeId, Vec<(OSMWayId, OSMCost)>>::default();

        let mut reader = osmio::read_pbf(map_file).ok()?;
        let mut num_actions = 0;
        for obj in reader.objects() {
            match obj {
                StringOSMObj::Node(node) => {
                    // `https://docs.rs/osmio/latest/osmio/trait.Node.html`
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
                    // `https://docs.rs/osmio/latest/osmio/trait.Way.html`
                    if way.num_nodes() < 2 {
                        continue;
                    }
                    // if way.deleted() {
                    //     continue;
                    // }

                    let action = OSMAction::from_osm_way(&way).unwrap();
                    // ways.insert(action.id, action);

                    if let Some(oneway) = way.tag("oneway")
                        && oneway == "yes"
                    {
                        num_actions += 1;
                        let last_node = OSMNodeId(OSMId(*way.nodes().last().unwrap()));
                        if let Some(a) = actions.get_mut(&last_node) {
                            a.push((action.id, action.cost));
                        } else {
                            actions.insert(last_node, vec![(action.id, action.cost)]);
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
                    // `https://docs.rs/osmio/latest/osmio/trait.Relation.html`
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
        let lon = Longitude(osmio::Lon::from_inner(123456));
        let lat_lon = (lat, lon);

        assert_eq!(OSMState::new(id, lat_lon), OSMState { id, lat, lon });
    }
}
