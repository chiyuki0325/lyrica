use std::collections::HashMap;
use zbus::zvariant::{Array, Dict, ObjectPath, OwnedValue};

#[derive(Debug, Clone)]
pub(crate) struct Metadata(pub HashMap<String, OwnedValue>);

impl Metadata {
    fn get_string(&self, key: &str) -> Option<String> {
        self.0
            .get(key)?
            .downcast_ref::<String>()
            .map_or(None, |s| Some(s.to_string()))
    }

    fn get_string_list(&self, key: &str) -> Option<Vec<String>> {
        self.0
            .get(key)?
            .downcast_ref::<Array>()
            .ok()?
            .iter()
            .filter_map(|it| it.downcast_ref::<String>().ok())
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .into()
    }

    fn get_object_path_as_string(&self, key: &str) -> Option<String> {
        self.0
            .get(key)?
            .downcast_ref::<ObjectPath>()
            .map_or(None, |s| Some(s.to_string()))
    }

    pub fn xesam_title(&self) -> Option<String> {
        self.get_string("xesam:title")
    }

    pub fn title(&self) -> Option<String> {
        self.xesam_title()
    }

    pub fn xesam_artist(&self) -> Option<Vec<String>> {
        self.get_string_list("xesam:artist")
    }

    pub fn artist(&self) -> Option<String> {
        self.xesam_artist().and_then(|artists| {
            if artists.is_empty() {
                None
            } else {
                Some(artists.join(", "))
            }
        })
    }

    pub fn xesam_url(&self) -> Option<String> {
        self.get_string("xesam:url")
    }

    pub fn mpris_art_url(&self) -> Option<String> {
        self.get_string("mpris:artUrl")
    }

    pub fn mpris_track_id(&self) -> Option<String> {
        self.get_object_path_as_string("mpris:trackid")
    }

    pub fn has_song_info(&self) -> bool {
        self.xesam_title().is_some() && self.xesam_artist().is_some()
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
