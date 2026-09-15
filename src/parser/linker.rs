use crate::common::instance::{Instance};


pub fn link_parent_references(instance: &mut Instance) {
    link_nodes_references(instance);
    link_clusters_references(instance);
}

fn link_nodes_references(instance: &mut Instance) {
    for subgroup in &mut instance.subgroups {
        for node_id in &subgroup.node_ids {
            instance.nodes[*node_id]
                .parent_subgroup_ids
                .insert(subgroup.id);
        }
    }
}

fn link_clusters_references(instance: &mut Instance) {
    for cluster in &instance.clusters {
        for subgroup_id in &cluster.subgroup_ids {
            instance.subgroups[*subgroup_id].parent_cluster_ids.insert(cluster.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::instance::{Cluster, Node, Subgroup};

    #[test]
    fn test_link_parent_references() {
        let mut instance = Instance {
            nodes: vec![
                Node { id: 0, ..Default::default() },
                Node { id: 1, ..Default::default() },
                Node { id: 2, ..Default::default() },
            ],
            subgroups: vec![
                Subgroup { id: 0, node_ids: vec![0, 1], ..Default::default() },
                Subgroup { id: 1, node_ids: vec![1, 2], ..Default::default() },
            ],
            clusters: vec![
                Cluster { id: 0, subgroup_ids: vec![0, 1] },
                Cluster { id: 1, subgroup_ids: vec![1] },
            ],
            ..Default::default()
        };

        link_parent_references(&mut instance);

        // Verify node -> subgroup parent links
        assert!(instance.nodes[0].parent_subgroup_ids.contains(&0));
        assert!(!instance.nodes[0].parent_subgroup_ids.contains(&1));

        assert!(instance.nodes[1].parent_subgroup_ids.contains(&0));
        assert!(instance.nodes[1].parent_subgroup_ids.contains(&1));

        assert!(!instance.nodes[2].parent_subgroup_ids.contains(&0));
        assert!(instance.nodes[2].parent_subgroup_ids.contains(&1));

        // Verify subgroup -> cluster parent links (including multi-parent subgroup 1)
        assert!(instance.subgroups[0].parent_cluster_ids.contains(&0));
        assert_eq!(instance.subgroups[0].parent_cluster_ids.len(), 1);

        assert!(instance.subgroups[1].parent_cluster_ids.contains(&0));
        assert!(instance.subgroups[1].parent_cluster_ids.contains(&1));
        assert_eq!(instance.subgroups[1].parent_cluster_ids.len(), 2);
    }
}