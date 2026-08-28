use super::edge::EdgeEvent;
use crate::flow::FLOW_MANAGER;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowPeer {
    pub peer_id: String,
    pub hostname: String,
    pub channel: u8,
    pub edge_relation: EdgeEvent,
}

impl FlowPeer {
    pub fn from_config_peer(p: &crate::config::FlowPeer) -> Option<Self> {
        let edge = if p.layout_x == -1 && p.layout_y == 0 {
            Some(EdgeEvent::Left)
        } else if p.layout_x == 1 && p.layout_y == 0 {
            Some(EdgeEvent::Right)
        } else if p.layout_x == 0 && p.layout_y == -1 {
            Some(EdgeEvent::Top)
        } else if p.layout_x == 0 && p.layout_y == 1 {
            Some(EdgeEvent::Bottom)
        } else {
            None
        };

        edge.map(|edge_relation| FlowPeer {
            peer_id: p.name.clone(),
            hostname: p.name.clone(),
            channel: p.channel_index,
            edge_relation,
        })
    }
}

pub fn resolve_peer(edge: EdgeEvent) -> Option<FlowPeer> {
    let config_peers = FLOW_MANAGER.flow_peers.read().unwrap();
    let resolved = config_peers.iter().filter(|p| p.paired).find_map(|p| {
        let fp = FlowPeer::from_config_peer(p)?;
        if fp.edge_relation == edge {
            Some(fp)
        } else {
            None
        }
    });

    if let Some(ref peer) = resolved {
        log::debug!(
            "[Topology] Resolved peer: peer_id={}, channel={}",
            peer.peer_id,
            peer.channel
        );
    } else {
        log::debug!("[Topology] No peer resolved for edge {:?}", edge);
    }

    resolved
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FlowPeer as ConfigPeer;

    #[test]
    fn test_topology_resolution_non_sequential() {
        // Create config peers with non-sequential channels and mismatched edge layouts:
        // Left   -> Channel 2 (peer_id: "left-pc")
        // Right  -> Channel 1 (peer_id: "right-pc")
        // Top    -> Channel 3 (peer_id: "top-pc")
        // Bottom -> Channel 0 (peer_id: "bottom-pc")
        let test_peers = [
            ConfigPeer {
                name: "right-pc".to_string(),
                ip: "192.168.1.101".to_string(),
                port: 50520,
                layout_x: 1,
                layout_y: 0,
                paired: true,
                fingerprint: "".to_string(),
                auto_reconnect: true,
                channel_index: 1,
            },
            ConfigPeer {
                name: "left-pc".to_string(),
                ip: "192.168.1.102".to_string(),
                port: 50520,
                layout_x: -1,
                layout_y: 0,
                paired: true,
                fingerprint: "".to_string(),
                auto_reconnect: true,
                channel_index: 2,
            },
            ConfigPeer {
                name: "bottom-pc".to_string(),
                ip: "192.168.1.103".to_string(),
                port: 50520,
                layout_x: 0,
                layout_y: 1,
                paired: true,
                fingerprint: "".to_string(),
                auto_reconnect: true,
                channel_index: 0,
            },
            ConfigPeer {
                name: "top-pc".to_string(),
                ip: "192.168.1.104".to_string(),
                port: 50520,
                layout_x: 0,
                layout_y: -1,
                paired: true,
                fingerprint: "".to_string(),
                auto_reconnect: true,
                channel_index: 3,
            },
        ];

        // 1. Verify mapping from config peer to FlowPeer
        let mapped: Vec<FlowPeer> = test_peers
            .iter()
            .flat_map(FlowPeer::from_config_peer)
            .collect();
        assert_eq!(mapped.len(), 4);

        // 2. Mock resolution by searching mapped slice using find (field-based lookup)
        let resolve_helper = |edge: EdgeEvent| -> Option<FlowPeer> {
            mapped.iter().find(|p| p.edge_relation == edge).cloned()
        };

        // Assert correct resolution by fields
        let peer_left = resolve_helper(EdgeEvent::Left).unwrap();
        assert_eq!(peer_left.peer_id, "left-pc");
        assert_eq!(peer_left.channel, 2);

        let peer_right = resolve_helper(EdgeEvent::Right).unwrap();
        assert_eq!(peer_right.peer_id, "right-pc");
        assert_eq!(peer_right.channel, 1);

        let peer_top = resolve_helper(EdgeEvent::Top).unwrap();
        assert_eq!(peer_top.peer_id, "top-pc");
        assert_eq!(peer_top.channel, 3);

        let peer_bottom = resolve_helper(EdgeEvent::Bottom).unwrap();
        assert_eq!(peer_bottom.peer_id, "bottom-pc");
        assert_eq!(peer_bottom.channel, 0);
    }
}
