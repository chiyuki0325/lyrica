use std::collections::HashMap;
use zbus::zvariant::{Array, Dict, OwnedValue};

#[derive(Debug)]
pub(super) struct Metadata(pub HashMap<String, OwnedValue>);

impl Metadata {
    pub fn title(&self) -> Option<String> {
        self.0
            .get("xesam:title")?
            .downcast_ref::<String>()
            .map_or(None, |s| Some(s))
    }

    pub fn artist(&self) -> Option<String> {
        Some(
            self.0
                .get("xesam:artist")?
                .downcast_ref::<Array>()
                .ok()?
                .iter()
                .filter_map(|it| it.downcast_ref::<String>().ok())
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

impl TryFrom<OwnedValue> for Metadata {
    type Error = zbus::Error;

    fn try_from(value: OwnedValue) -> Result<Self, Self::Error> {
        let metadata_dict = value.downcast_ref::<Dict>()?;
        let metadata: HashMap<String, OwnedValue> = metadata_dict.try_into()?;
        Ok(Metadata(metadata))
    }
}
