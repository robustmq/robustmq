use raft_proto::eraftpb::{Entry, EntryType, ConfState, HardState};
use serde::{Deserialize, Serialize};
use crate::errors::MetaError;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SaveRDSHardState {
    pub term: u64,
    pub vote: u64,
    pub commit: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SaveRDSConfState {
    pub voters: Vec<u64>,
    pub learners: Vec<u64>,
    pub voters_outgoing: Vec<u64>,
    pub learners_next: Vec<u64>,
    pub auto_leave: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SaveRDSEntry {
    pub entry_type: u64,
    pub term: u64,
    pub index: u64,
    pub data: ::bytes::Bytes,
    pub context: ::bytes::Bytes,
    pub sync_log: bool,
}

/// Converts data of type SaveRDSEntry to raft::prelude::Entry
pub fn convert_entry_from_rds_save_entry(rds_entry: SaveRDSEntry) -> Result<Entry, MetaError> {
    let mut ent = Entry::default();

    //What if the Entry Enum options change, there may be a Bug
    let et = match rds_entry.entry_type {
        0_u64 => EntryType::EntryNormal,
        1_u64 => EntryType::EntryConfChange,
        2_u64 => EntryType::EntryConfChangeV2,
        _ => EntryType::EntryNormal,
    };

    ent.entry_type = et;
    ent.term = rds_entry.term;
    ent.index = rds_entry.index;
    ent.data = rds_entry.data;
    ent.context = rds_entry.context;
    ent.sync_log = rds_entry.sync_log;
    return Ok(ent);
}

/// Converts data of type SaveRDSConfState to raft::prelude::ConfState
pub fn convert_conf_state_from_rds_cs(scs: SaveRDSConfState) -> ConfState {
    let mut cs = ConfState::default();
    cs.voters = scs.voters;
    cs.learners = scs.learners;
    cs.voters_outgoing = scs.voters_outgoing;
    cs.learners_next = scs.learners_next;
    cs.auto_leave = scs.auto_leave;
    return cs;
}

/// Converts data of type SaveRDSHardState to raft::prelude::HardState
pub fn convert_hard_state_from_rds_hs(shs: SaveRDSHardState) -> HardState {
    let mut hs = HardState::default();
    hs.term = shs.term;
    hs.vote = shs.vote;
    hs.commit = shs.commit;
    return hs;
}
