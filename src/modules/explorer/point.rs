use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::common::Point;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExplorerPoint {
    pub inner: Point,
    pub selected: bool,
}

impl Deref for ExplorerPoint {
    type Target = Point;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ExplorerPoint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Serialize for ExplorerPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'_a> Deserialize<'_a> for ExplorerPoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'_a>,
    {
        let inner = Point::deserialize(deserializer)?;
        Ok(Self {
            inner,
            selected: false,
        })
    }
}
