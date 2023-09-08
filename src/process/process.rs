use crate::process::{child, init, parent};

pub enum Process {
    Parent(parent::ParentProcess),
    Child(child::ChildProcess),
    Init(init::InitProcess),
}
