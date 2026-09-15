use std::{collections::HashSet, fmt};

#[derive(Debug, Clone, Default)]
pub enum Metric {
    Euc2d,
    #[default]
    Euc3d,
    Man2d,
    Man3d,
}

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3 {
    pub fn distance_to(&self, other: &Point3, metric: &Metric) -> f64 {
        match metric {
            Metric::Euc2d => self.distance_euc_2d(other),
            Metric::Euc3d => self.distance_euc_3d(other),
            Metric::Man2d => self.distance_man_2d(other),
            Metric::Man3d => self.distance_man_3d(other),
        }
    }

    fn distance_euc_2d(&self, other: &Point3) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    fn distance_euc_3d(&self, other: &Point3) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
            .sqrt()
    }

    fn distance_man_2d(&self, other: &Point3) -> f64 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    fn distance_man_3d(&self, other: &Point3) -> f64 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Node {
    pub id: usize,
    pub point: Point3,
    pub parent_subgroup_ids: HashSet<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct Subgroup {
    pub id: usize,
    pub profit: f64,
    pub node_ids: Vec<usize>,
    pub parent_cluster_ids: HashSet<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct Cluster {
    pub id: usize,
    pub subgroup_ids: Vec<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct Vehicle {
    pub id: usize,
    pub budget: f64,
    pub start_node_id: usize,
    pub end_node_id: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Instance {
    pub folder_path: String,
    pub name: String,
    pub metric: Metric,
    pub nodes: Vec<Node>,
    pub subgroups: Vec<Subgroup>,
    pub clusters: Vec<Cluster>,
    pub vehicles: Vec<Vehicle>,
}

impl Instance {
    pub fn get_distance(&self, from_id: usize, to_id: usize) -> f64 {
        self.nodes[from_id]
            .point
            .distance_to(&self.nodes[to_id].point, &self.metric)
    }
}

pub trait HasId {
    fn id(&self) -> usize;
}

impl HasId for Node {
    fn id(&self) -> usize {
        self.id
    }
}

impl HasId for Subgroup {
    fn id(&self) -> usize {
        self.id
    }
}

impl HasId for Cluster {
    fn id(&self) -> usize {
        self.id
    }
}

impl HasId for Vehicle {
    fn id(&self) -> usize {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_display_and_default() {
        assert_eq!(Metric::default().to_string(), "Euc3d");
        assert_eq!(Metric::Euc2d.to_string(), "Euc2d");
        assert_eq!(Metric::Euc3d.to_string(), "Euc3d");
        assert_eq!(Metric::Man2d.to_string(), "Man2d");
        assert_eq!(Metric::Man3d.to_string(), "Man3d");
    }

    #[test]
    fn test_point3_distances() {
        let p1 = Point3 { x: 0.0, y: 0.0, z: 0.0 };
        let p2 = Point3 { x: 3.0, y: 4.0, z: 12.0 };

        assert_eq!(p1.distance_to(&p2, &Metric::Euc2d), 5.0);
        assert_eq!(p1.distance_to(&p2, &Metric::Euc3d), 13.0);
        assert_eq!(p1.distance_to(&p2, &Metric::Man2d), 7.0);
        assert_eq!(p1.distance_to(&p2, &Metric::Man3d), 19.0);
    }

    #[test]
    fn test_instance_get_distance() {
        let instance = Instance {
            metric: Metric::Euc2d,
            nodes: vec![
                Node { id: 0, point: Point3 { x: 0.0, y: 0.0, z: 0.0 }, ..Default::default() },
                Node { id: 1, point: Point3 { x: 6.0, y: 8.0, z: 0.0 }, ..Default::default() },
            ],
            ..Default::default()
        };

        assert_eq!(instance.get_distance(0, 1), 10.0);
    }

    #[test]
    fn test_has_id_implementations() {
        let node = Node { id: 42, ..Default::default() };
        let subgroup = Subgroup { id: 10, ..Default::default() };
        let cluster = Cluster { id: 5, ..Default::default() };
        let vehicle = Vehicle { id: 3, ..Default::default() };

        assert_eq!(node.id(), 42);
        assert_eq!(subgroup.id(), 10);
        assert_eq!(cluster.id(), 5);
        assert_eq!(vehicle.id(), 3);
    }
}

