use zbus::proxy;

#[proxy(
    interface = "org.freedesktop.DBus",
    default_service = "org.freedesktop.DBus",
    default_path = "/org/freedesktop/DBus"
)]
pub(crate) trait DBus {
    #[zbus(signal)]
    fn name_owner_changed(&self, name: &str, old_owner: &str, new_owner: &str) -> zbus::Result<()>;
}
