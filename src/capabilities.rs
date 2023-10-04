use crate::spec::{LinuxCapabilities, LinuxCapabilityType};
use caps::*;

use anyhow::{Ok, Result};

fn set_cap_hash_set(caps: &[LinuxCapabilityType]) -> CapsHashSet {
    let mut capabilities = CapsHashSet::new();
    for c in caps {
        capabilities.insert(c.cap);
    }
    capabilities
}

pub fn reset_effective() -> Result<()> {
    log::debug!("reset all caps");
    // permitted capabilities are all the capabilities that we are allowed to acquire
    let permitted = caps::read(None, CapSet::Permitted)?;
    set(None, CapSet::Effective, &permitted)?;
    Ok(())
}

pub fn set_capabilities(cs: &LinuxCapabilities) -> Result<()> {
    let all = caps::all();
    log::debug!("dropping bounding capabilities to {:?}", cs.bounding);
    for c in all.difference(&set_cap_hash_set(&cs.bounding)) {
        caps::drop(None, CapSet::Bounding, *c)?;
    }
    set(None, CapSet::Effective, &set_cap_hash_set(&cs.effective))?;
    set(None, CapSet::Permitted, &set_cap_hash_set(&cs.permitted))?;
    set(
        None,
        CapSet::Inheritable,
        &set_cap_hash_set(&cs.inheritable),
    )?;
    set(None, CapSet::Ambient, &set_cap_hash_set(&cs.ambient))?;

    Ok(())
}
