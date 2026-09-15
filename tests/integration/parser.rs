use std::collections::HashSet;
use std::path::Path;
use tcops::parser::load_instance;

#[test]
fn test_load_burma14_instance() {
    let path = Path::new("resource/tcops/3/burma14.tcops");
    let instance = load_instance(path).expect("Failed to parse burma14.tcops");

    assert_eq!(instance.name, "burma14");
    assert_eq!(instance.nodes.len(), 14);
    assert_eq!(instance.subgroups.len(), 4);
    assert_eq!(instance.clusters.len(), 2);
    assert_eq!(instance.vehicles.len(), 3);

    // Verify subgroup node assignments
    assert_eq!(instance.subgroups[0].node_ids, vec![0]);
    assert_eq!(instance.subgroups[1].node_ids, vec![2, 3, 13]);
    assert_eq!(instance.subgroups[2].node_ids, vec![1, 7, 8, 9, 10]);
    assert_eq!(instance.subgroups[3].node_ids, vec![4, 5, 6, 11, 12]);

    // Verify cluster subgroup assignments
    assert_eq!(instance.clusters[0].subgroup_ids, vec![0]);
    assert_eq!(instance.clusters[1].subgroup_ids, vec![1, 2, 3]);

    // Verify parent links for node 3 and node 7
    assert!(instance.nodes[3].parent_subgroup_ids.contains(&1));
    assert!(instance.nodes[7].parent_subgroup_ids.contains(&2));

    // Verify parent cluster IDs
    assert_eq!(instance.subgroups[0].parent_cluster_ids, HashSet::from([0]));
    assert_eq!(instance.subgroups[1].parent_cluster_ids, HashSet::from([1]));
    assert_eq!(instance.subgroups[2].parent_cluster_ids, HashSet::from([1]));
    assert_eq!(instance.subgroups[3].parent_cluster_ids, HashSet::from([1]));
}
