pub const ILIAS_URL: &str = "https://ilias.studium.kit.edu";

#[macro_export]
macro_rules! ilias_url {
    ($id:tt) => {
        format!("https://ilias.studium.kit.edu/goto.php?target=exc_{}&client_id=produktiv", $id)
    };
}