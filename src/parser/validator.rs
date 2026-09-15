use crate::common::instance::{HasId, Instance, Node, Subgroup};
use std::io::{Error, ErrorKind};


pub fn validate_section_data_id(section: &str, id: usize, last_id: isize) -> Result<(), Error> {

    if id > 0 && last_id + 1 == 0 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("The {} id must start with 0", section),
        ));
    }

    if id as isize != last_id + 1 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("{} id {} need be sequential", section, id),
        ));
    }

    Ok(())
} 

pub fn validate_item_id<T>(container_name: &str, container: &[T], item_id: usize) -> Result<(), Error>
where
    T: HasId,
{
    let item = container.get(item_id).ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidData,
            format!("Integrity error: {} ID {} does not exist.", container_name, item_id),
        )
    })?;

    if item.id() != item_id {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Integrity error: {} ID {} does not exist.", container_name, item_id),
        ));
    }

    Ok(())
}

pub fn validate_node_has_subgroup(nodes: &[Node]) -> Result<(), Error> {
    let unassigned_node_ids: Vec<usize> = nodes
        .iter()
        .filter(|n| n.parent_subgroup_ids.is_empty())
        .map(|n| n.id)
        .collect();

    if let Some(&first_id) = unassigned_node_ids.first() {
        if unassigned_node_ids.len() == 1 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Integrity error: Node ID {} does not belong to any subgroup.", first_id),
            ));
        } else {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Integrity error: Node ID {} does not belong to any subgroup (all unassigned nodes: {:?}).",
                    first_id, unassigned_node_ids
                ),
            ));
        }
    }

    Ok(())
}

pub fn validate_subgroup_has_cluster(subgroups: &[Subgroup]) -> Result<(), Error> {
    let unassigned_subgroup_ids: Vec<usize> = subgroups
        .iter()
        .filter(|s| s.parent_cluster_ids.is_empty())
        .map(|s| s.id)
        .collect();

    if let Some(&first_id) = unassigned_subgroup_ids.first() {
        if unassigned_subgroup_ids.len() == 1 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Integrity error: Subgroup ID {} does not belong to any cluster.", first_id),
            ));
        } else {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Integrity error: Subgroup ID {} does not belong to any cluster (all unassigned subgroups: {:?}).",
                    first_id, unassigned_subgroup_ids
                ),
            ));
        }
    }

    Ok(())
}

pub fn validate_hierarchy(instance: &Instance) -> Result<(), Error> {
    validate_node_has_subgroup(&instance.nodes)?;
    validate_subgroup_has_cluster(&instance.subgroups)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyItem {
        id: usize,
    }

    impl HasId for DummyItem {
        fn id(&self) -> usize {
            self.id
        }
    }

    #[test]
    fn test_validate_section_data_id() {
        // First item starting at 0
        assert!(validate_section_data_id("Node", 0, -1).is_ok());

        // First item starting at 1 (should fail)
        let err = validate_section_data_id("Node", 1, -1);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("The Node id must start with 0"));

        // Sequential items
        assert!(validate_section_data_id("Node", 1, 0).is_ok());
        assert!(validate_section_data_id("Node", 2, 1).is_ok());

        // Non-sequential items
        let err = validate_section_data_id("Node", 3, 1);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Node id 3 need be sequential"));

        // Duplicate item
        let err = validate_section_data_id("Node", 1, 1);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Node id 1 need be sequential"));
    }

    #[test]
    fn test_validate_item_id() {
        let items = vec![DummyItem { id: 0 }, DummyItem { id: 1 }];

        // Valid item IDs
        assert!(validate_item_id("Node", &items, 0).is_ok());
        assert!(validate_item_id("Node", &items, 1).is_ok());

        // Out of bounds ID
        let err = validate_item_id("Node", &items, 2);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Integrity error: Node ID 2 does not exist."));

        // Item at index 0 but id is 99
        let mismatched = vec![DummyItem { id: 99 }];
        let err = validate_item_id("Node", &mismatched, 0);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Integrity error: Node ID 0 does not exist."));
    }

    #[test]
    fn test_validate_node_has_subgroup_success() {
        use std::collections::HashSet;

        let nodes = vec![
            Node {
                id: 0,
                parent_subgroup_ids: HashSet::from([0]),
                ..Default::default()
            },
            Node {
                id: 1,
                parent_subgroup_ids: HashSet::from([0, 1]),
                ..Default::default()
            },
        ];

        assert!(validate_node_has_subgroup(&nodes).is_ok());
    }

    #[test]
    fn test_validate_node_has_subgroup_failure_single() {
        use std::collections::HashSet;

        let nodes = vec![
            Node {
                id: 0,
                parent_subgroup_ids: HashSet::from([0]),
                ..Default::default()
            },
            Node {
                id: 6,
                parent_subgroup_ids: HashSet::new(),
                ..Default::default()
            },
        ];

        let err = validate_node_has_subgroup(&nodes);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Integrity error: Node ID 6 does not belong to any subgroup."));
    }

    #[test]
    fn test_validate_node_has_subgroup_failure_multiple() {
        use std::collections::HashSet;

        let nodes = vec![
            Node {
                id: 0,
                parent_subgroup_ids: HashSet::from([0]),
                ..Default::default()
            },
            Node {
                id: 3,
                parent_subgroup_ids: HashSet::new(),
                ..Default::default()
            },
            Node {
                id: 6,
                parent_subgroup_ids: HashSet::new(),
                ..Default::default()
            },
        ];

        let err = validate_node_has_subgroup(&nodes);
        assert!(err.is_err());
        let msg = err.unwrap_err().to_string();
        assert!(msg.contains("Integrity error: Node ID 3 does not belong to any subgroup"));
        assert!(msg.contains("[3, 6]"));
    }

    #[test]
    fn test_validate_subgroup_has_cluster_success() {
        use std::collections::HashSet;

        let subgroups = vec![
            Subgroup {
                id: 0,
                parent_cluster_ids: HashSet::from([0]),
                ..Default::default()
            },
            Subgroup {
                id: 1,
                parent_cluster_ids: HashSet::from([0, 1]),
                ..Default::default()
            },
        ];

        assert!(validate_subgroup_has_cluster(&subgroups).is_ok());
    }

    #[test]
    fn test_validate_subgroup_has_cluster_failure_single() {
        use std::collections::HashSet;

        let subgroups = vec![
            Subgroup {
                id: 0,
                parent_cluster_ids: HashSet::from([0]),
                ..Default::default()
            },
            Subgroup {
                id: 2,
                parent_cluster_ids: HashSet::new(),
                ..Default::default()
            },
        ];

        let err = validate_subgroup_has_cluster(&subgroups);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Integrity error: Subgroup ID 2 does not belong to any cluster."));
    }

    #[test]
    fn test_validate_subgroup_has_cluster_failure_multiple() {
        use std::collections::HashSet;

        let subgroups = vec![
            Subgroup {
                id: 0,
                parent_cluster_ids: HashSet::new(),
                ..Default::default()
            },
            Subgroup {
                id: 1,
                parent_cluster_ids: HashSet::new(),
                ..Default::default()
            },
        ];

        let err = validate_subgroup_has_cluster(&subgroups);
        assert!(err.is_err());
        let msg = err.unwrap_err().to_string();
        assert!(msg.contains("Integrity error: Subgroup ID 0 does not belong to any cluster"));
        assert!(msg.contains("[0, 1]"));
    }

    #[test]
    fn test_validate_hierarchy() {
        use std::collections::HashSet;

        let mut instance = Instance {
            nodes: vec![
                Node { id: 0, parent_subgroup_ids: HashSet::from([0]), ..Default::default() },
            ],
            subgroups: vec![
                Subgroup { id: 0, parent_cluster_ids: HashSet::from([0]), ..Default::default() },
            ],
            ..Default::default()
        };

        assert!(validate_hierarchy(&instance).is_ok());

        // Subgroup without cluster
        instance.subgroups[0].parent_cluster_ids.clear();
        assert!(validate_hierarchy(&instance).is_err());

        // Node without subgroup
        instance.subgroups[0].parent_cluster_ids.insert(0);
        instance.nodes[0].parent_subgroup_ids.clear();
        assert!(validate_hierarchy(&instance).is_err());
    }
}