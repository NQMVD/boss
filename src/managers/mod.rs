pub mod apt;
pub mod cargo;
pub mod go;
pub mod snap;
pub mod yay;

pub use apt::check_apt;
pub use cargo::check_cargo;
// pub use go::check_go;
pub use snap::check_snap;
pub use yay::check_yay;
