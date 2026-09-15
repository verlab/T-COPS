use crate::common::instance::{Node, Subgroup};
use crate::parser::sections::common::handle_section;
use crate::parser::utils::{LineTracker, parse_integer, parse_float};
use crate::parser::validator::validate_item_id;
use std::collections::HashSet;
use std::io::{BufRead, Error, ErrorKind};

const SUBGROUP_DATA_MIN_PARTS: usize = 2;

pub fn process<R: BufRead>(
    tracker: &mut LineTracker<R>,
    subgroups: &mut Vec<Subgroup>,
    nodes: &[Node],
) -> Result<(), Error> {
    handle_section(tracker, subgroups, "Subgroup", |parts| {
        parse(parts, nodes)
    })
}

fn parse(parts: Vec<&str>, nodes: &[Node]) -> Result<Subgroup, Error> {
    if parts.len() < SUBGROUP_DATA_MIN_PARTS {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("Invalid subgroup data: {:?}", parts),
        ));
    }

    let id = parse_integer(parts[0])?;
    let profit = parse_float(parts[1])?;
    let mut node_ids = Vec::new();
    for part in &parts[2..] {
        let node_id = parse_integer(part)?;

        validate_item_id("Node", nodes, node_id)?;
        node_ids.push(node_id);
    }

    Ok(Subgroup {
        id,
        profit,
        node_ids,
        parent_cluster_ids: HashSet::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_parse_subgroup_success() {
        let nodes = vec![
            Node { id: 0, ..Default::default() },
            Node { id: 1, ..Default::default() },
        ];

        let sg = parse(vec!["0", "100.5", "0", "1"], &nodes).unwrap();
        assert_eq!(sg.id, 0);
        assert_eq!(sg.profit, 100.5);
        assert_eq!(sg.node_ids, vec![0, 1]);
    }

    #[test]
    fn test_parse_subgroup_invalid_node_ref() {
        let nodes = vec![Node { id: 0, ..Default::default() }];

        let err = parse(vec!["0", "100.5", "0", "5"], &nodes);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Integrity error: Node ID 5 does not exist."));
    }

    #[test]
    fn test_parse_subgroup_insufficient_parts() {
        let nodes = vec![];
        let err = parse(vec!["0"], &nodes);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Invalid subgroup data"));
    }

    #[test]
    fn test_process_subgroups() {
        let nodes = vec![
            Node { id: 0, ..Default::default() },
            Node { id: 1, ..Default::default() },
        ];
        let input = "0 50.0 0 1\n";
        let mut tracker = LineTracker::new(Cursor::new(input));
        let mut subgroups = Vec::with_capacity(1);

        assert!(process(&mut tracker, &mut subgroups, &nodes).is_ok());
        assert_eq!(subgroups.len(), 1);
        assert_eq!(subgroups[0].id, 0);
        assert_eq!(subgroups[0].node_ids, vec![0, 1]);
    }
}

