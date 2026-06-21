use super::validate_node_ports;

#[test]
fn node_port_validation_rejects_zero_and_duplicate_ports() {
    assert!(validate_node_ports(0, 10333, None).is_err());
    assert!(validate_node_ports(10332, 0, None).is_err());
    assert!(validate_node_ports(10332, 10333, Some(0)).is_err());
    assert!(validate_node_ports(10332, 10332, None).is_err());
    assert!(validate_node_ports(10332, 10333, Some(10332)).is_err());
    assert!(validate_node_ports(10332, 10333, Some(10333)).is_err());
}

#[test]
fn node_port_validation_accepts_distinct_nonzero_ports() {
    assert!(validate_node_ports(10332, 10333, None).is_ok());
    assert!(validate_node_ports(10332, 10333, Some(10334)).is_ok());
}
