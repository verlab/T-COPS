mod header;
mod linker;
mod sections;
mod utils;
mod validator;

use crate::common::instance::Instance;
use std::fs::File;
use std::io::{BufReader, Error};
use std::path::Path;

pub fn load_instance(path: &Path) -> Result<Instance, Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut tracker = utils::LineTracker::new(reader);
    let mut instance = Instance::default();

    header::read_header(&mut instance, &mut tracker)
        .map_err(|e| format_err(tracker.current_line(), e))?;

    sections::read_sections(&mut instance, &mut tracker)
        .map_err(|e| format_err(tracker.current_line(), e))?;

    linker::link_parent_references(&mut instance);

    validator::validate_hierarchy(&instance)?;

    let input_folder_path = Path::new(&path);
    let input_folder_path = input_folder_path.parent().unwrap_or(Path::new("./"));

    let folder_path = match input_folder_path.to_str() {
        Some(path) => path,
        None => {
            eprintln!("Failed to convert input folder path to string");
            std::process::exit(1);
        }
    };

    instance.folder_path = folder_path.to_string();

    Ok(instance)
}

fn format_err(line_num: usize, e: Error) -> Error {
    Error::new(e.kind(), format!("Error on line {}: {}", line_num, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_format_err() {
        let original_err = Error::new(std::io::ErrorKind::InvalidData, "test error");
        let formatted = format_err(42, original_err);
        assert_eq!(formatted.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(formatted.to_string(), "Error on line 42: test error");
    }

    #[test]
    fn test_load_instance_file_not_found() {
        let path = Path::new("non_existent_file_path_12345.tcops");
        let result = load_instance(path);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn test_load_instance_valid_file() {
        let content = "NAME: test_inst
TYPE: TCOPS
COMMENT: comment
DIMENSION: 2
SUBGROUPS: 1
CLUSTERS: 1
VEHICLES: 1
EDGE_WEIGHT_TYPE: EUC_2D
NODE_COORD_SECTION: id profit x y
0 0.0 1.0 2.0
1 5.0 3.0 4.0
SUBGROUP_SECTION: subgroup_id id-vertex-list
0 10.0 0 1
CLUSTER_SECTION: cluster_id id-subgroup-list
0 0
VEHICLES_SECTION: id tmax start end
0 50.0 0 1
";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();

        let instance = load_instance(temp_file.path()).unwrap();
        assert_eq!(instance.name, "test_inst");
        assert_eq!(instance.nodes.len(), 2);
        assert_eq!(instance.subgroups.len(), 1);
        assert_eq!(instance.clusters.len(), 1);
        assert_eq!(instance.vehicles.len(), 1);

        // Verify linker results
        assert!(instance.nodes[0].parent_subgroup_ids.contains(&0));
        assert!(instance.nodes[1].parent_subgroup_ids.contains(&0));
        assert!(instance.subgroups[0].parent_cluster_ids.contains(&0));

        // Verify folder_path
        let expected_folder = temp_file.path().parent().unwrap().to_str().unwrap();
        assert_eq!(instance.folder_path, expected_folder);
    }

    #[test]
    fn test_load_instance_invalid_format_line_num() {
        let content = "NAME: test_inst
TYPE: TCOPS
COMMENT: comment
DIMENSION: 2
SUBGROUPS: 1
CLUSTERS: 1
VEHICLES: 1
EDGE_WEIGHT_TYPE: EUC_2D
NODE_COORD_SECTION: id profit x y
0 0.0 1.0 2.0
INVALID_LINE_WITHOUT_ID
";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();

        let result = load_instance(temp_file.path());
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("Error on line 11:"));
    }

    #[test]
    fn test_load_instance_node_without_subgroup_fails() {
        let content = "NAME: test_inst
TYPE: TCOPS
COMMENT: Node 1 is not in any subgroup
DIMENSION: 2
SUBGROUPS: 1
CLUSTERS: 1
VEHICLES: 1
EDGE_WEIGHT_TYPE: EUC_2D
NODE_COORD_SECTION: id profit x y
0 0.0 1.0 2.0
1 5.0 3.0 4.0
SUBGROUP_SECTION: subgroup_id profit id-vertex-list
0 10.0 0
CLUSTER_SECTION: cluster_id id-subgroup-list
0 0
VEHICLES_SECTION: id tmax start end
0 50.0 0 0
";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();

        let result = load_instance(temp_file.path());
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("Integrity error: Node ID 1 does not belong to any subgroup."));
    }

    #[test]
    fn test_load_instance_subgroup_without_cluster_fails() {
        let content = "NAME: test_inst
TYPE: TCOPS
COMMENT: Subgroup 1 is not in any cluster
DIMENSION: 2
SUBGROUPS: 2
CLUSTERS: 1
VEHICLES: 1
EDGE_WEIGHT_TYPE: EUC_2D
NODE_COORD_SECTION: id profit x y
0 0.0 1.0 2.0
1 5.0 3.0 4.0
SUBGROUP_SECTION: subgroup_id profit id-vertex-list
0 10.0 0
1 20.0 1
CLUSTER_SECTION: cluster_id id-subgroup-list
0 0
VEHICLES_SECTION: id tmax start end
0 50.0 0 0
";
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();

        let result = load_instance(temp_file.path());
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("Integrity error: Subgroup ID 1 does not belong to any cluster."));
    }

    #[test]
    fn test_load_instance_teste_tcops() {
        let path = Path::new("teste.tcops");
        if path.exists() {
            let result = load_instance(path);
            assert!(result.is_ok());
            let instance = result.unwrap();
            assert_eq!(instance.nodes.len(), 7);
            assert_eq!(instance.subgroups.len(), 6);
            assert_eq!(instance.clusters.len(), 5);
        }
    }
}